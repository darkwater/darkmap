use std::collections::HashMap;

use geo::{Coord, LineString, MultiPolygon, Point, Polygon, Rect};
use lazy_static::lazy_static;
use serde::Deserialize;
use thiserror::Error;

lazy_static! {
    static ref CLIENT: surf::Client = surf::Client::new();
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to download data (status: {status})")]
    LoadError {
        error: anyhow::Error,
        status: surf::http::StatusCode,
    },
    #[error("Failed to parse data")]
    ParseError(#[from] serde_json::Error),
}

impl From<surf::Error> for Error {
    fn from(value: surf::Error) -> Self {
        Self::LoadError {
            status: value.status(),
            error: value.into_inner(),
        }
    }
}

pub async fn load(query: &str) -> Result<ApiResponse, Error> {
    let body = format!("data={}", urlencoding::encode(query));

    let mut res = CLIENT
        .post("https://overpass-api.de/api/interpreter")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .body(body)
        .await?;

    Ok(res.body_json::<ApiResponse>().await?)
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub version: f64,
    pub generator: String,
    pub osm3s: Osm3s,
    pub elements: Vec<Element>,
}

#[derive(Debug, Deserialize)]
pub struct Osm3s {
    pub timestamp_osm_base: Option<String>,
    pub timestamp_areas_base: Option<String>,
    pub timestamp_osm_max: Option<String>,
    pub timestamp_areas_max: Option<String>,
    pub api_status: Option<String>,
    pub copyright: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Element {
    #[serde(rename = "node")]
    Node(Node),
    #[serde(rename = "way")]
    Way(Way),
    #[serde(rename = "relation")]
    Relation(Relation),
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: i64,
    #[serde(flatten)]
    #[serde(with = "point")]
    pub point: Point,
    pub tags: Tags,
}

#[derive(Debug, Deserialize)]
pub struct Way {
    pub id: i64,
    #[serde(with = "rect")]
    pub bounds: Rect,
    pub nodes: Vec<i64>,
    #[serde(with = "vec_coord")]
    pub geometry: Vec<Coord>,
    pub tags: Tags,
}

impl Way {
    pub fn is_closed(&self) -> bool {
        self.nodes.first() == self.nodes.last()
    }

    pub fn polygon(&self) -> Option<Polygon> {
        if self.is_closed() {
            Some(Polygon::new(self.geometry.clone().into(), vec![]))
        } else {
            None
        }
    }

    pub fn multiline(&self) -> Option<LineString> {
        if self.nodes.len() > 1 {
            Some(LineString::new(self.geometry.clone()))
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Bounds {
    pub minlat: f64,
    pub minlon: f64,
    pub maxlat: f64,
    pub maxlon: f64,
}

impl Bounds {
    pub fn center(&self) -> Point {
        Point::new((self.minlon + self.maxlon) / 2., (self.minlat + self.maxlat) / 2.)
    }

    pub fn north(&self) -> f64 {
        self.maxlat
    }

    pub fn east(&self) -> f64 {
        self.maxlon
    }

    pub fn south(&self) -> f64 {
        self.minlat
    }

    pub fn west(&self) -> f64 {
        self.minlon
    }

    pub fn north_west(&self) -> Point {
        Point::new(self.west(), self.north())
    }

    pub fn north_east(&self) -> Point {
        Point::new(self.east(), self.north())
    }

    pub fn south_west(&self) -> Point {
        Point::new(self.west(), self.south())
    }

    pub fn south_east(&self) -> Point {
        Point::new(self.east(), self.south())
    }
}

#[derive(Debug, Deserialize)]
pub struct Relation {
    pub id: i64,
    #[serde(with = "opt_rect")]
    pub bounds: Option<Rect>,
    pub members: Vec<Element>,
    pub tags: Tags,
}

impl Relation {
    pub fn polygon(&self) -> Option<MultiPolygon> {
        let mut outers: Vec<LineString> = vec![];

        for member in &self.members {
            if let Element::Way(way) = member {
                if let Some(multiline) = way.multiline() {
                    if let Some(cont) = outers
                        .iter_mut()
                        .find(|o| o.coords().last() == multiline.coords().next())
                    {
                        let coords = cont
                            .coords()
                            .chain(multiline.coords().skip(1))
                            .copied()
                            .collect();

                        *cont = LineString::new(coords);
                    } else {
                        outers.push(multiline)
                    }
                }
            }
        }

        let _outers: Vec<Polygon> = outers
            .into_iter()
            .filter(|ls| ls.is_closed())
            .map(|ls| Polygon::new(ls, vec![]))
            .collect();

        todo!()
    }
}

// #[derive(Debug, Deserialize)]
// pub enum Member {
//     #[serde(rename = "node")]
//     Node(NodeMember),
//     #[serde(rename = "way")]
//     Way(WayMember),
//     #[serde(rename = "relation")]
//     Relation(RelationMember),
// }

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Tags(pub HashMap<String, String>);

impl Tags {
    pub fn building_height(&self) -> Option<f32> {
        if let Some(height) = self.0.get("height") {
            height.split_whitespace().next().unwrap().parse().ok()
        } else if let Some(levels) = self.0.get("building:levels") {
            levels.parse::<f32>().ok().map(|l| l * 3.)
        } else {
            None
        }
    }

    pub fn road_width(&self) -> f32 {
        if let Some(width) = self
            .0
            .get("width")
            .and_then(|w| w.split_whitespace().next())
            .and_then(|w| w.parse().ok())
        {
            width
        } else if let Some(role) = self.0.get("highway") {
            let lanes = 1.;
            // let lanes = self
            //     .0
            //     .get("lanes")
            //     .and_then(|l| l.parse::<f32>().ok())
            //     .unwrap_or(1.);

            // lane width
            let lane_width = match role.as_str() {
                "motorway" => 6.,
                "trunk" => 5.,
                "primary" => 4.,
                "secondary" => 3.5,
                "tertiary" => 3.,
                "residential" => 2.75,
                "service" => 2.5,
                "unclassified" => 2.5,
                "cycleway" => 2.5,
                "footway" => 1.5,
                _ => 2.5,
            };

            lanes * lane_width
        } else {
            2.5
        }
    }
}

mod point {
    use geo::Point;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Point, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct LatLon {
            lat: f64,
            lon: f64,
        }

        let LatLon { lat, lon } = LatLon::deserialize(deserializer)?;

        Ok(Point::new(lon, lat))
    }
}

mod vec_coord {
    use geo::Coord;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Coord>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct LatLon {
            lat: f64,
            lon: f64,
        }

        let points: Vec<LatLon> = Vec::deserialize(deserializer)?;

        Ok(points
            .into_iter()
            .map(|p| Coord { x: p.lon, y: p.lat })
            .collect::<Vec<Coord>>())
    }
}

mod rect {
    use geo::Rect;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Rect, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Bounds {
            minlat: f64,
            minlon: f64,
            maxlat: f64,
            maxlon: f64,
        }

        let Bounds { minlat, minlon, maxlat, maxlon } = Bounds::deserialize(deserializer)?;

        Ok(Rect::new((minlon, minlat), (maxlon, maxlat)))
    }
}

mod opt_rect {
    use geo::Rect;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Rect>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Bounds {
            minlat: f64,
            minlon: f64,
            maxlat: f64,
            maxlon: f64,
        }

        Ok(Option::<Bounds>::deserialize(deserializer)?.map(
            |Bounds { minlat, minlon, maxlat, maxlon }| {
                Rect::new((minlon, minlat), (maxlon, maxlat))
            },
        ))
    }
}
