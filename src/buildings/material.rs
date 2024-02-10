use bevy::prelude::*;
use catppuccin::Colour;

use crate::COLORS;

#[derive(Resource)]
pub struct Materials {
    pub default: Handle<StandardMaterial>,
    pub residential: Handle<StandardMaterial>,
    pub outbuilding: Handle<StandardMaterial>,
    pub agricultural: Handle<StandardMaterial>,
    pub industrial: Handle<StandardMaterial>,
    pub commercial: Handle<StandardMaterial>,
    pub school: Handle<StandardMaterial>,
    pub retail: Handle<StandardMaterial>,
    pub construction: Handle<StandardMaterial>,
    pub service: Handle<StandardMaterial>,
    pub hotel: Handle<StandardMaterial>,
    pub warehouse: Handle<StandardMaterial>,
    pub office: Handle<StandardMaterial>,
    pub civic: Handle<StandardMaterial>,
    pub hospital: Handle<StandardMaterial>,
    pub religious: Handle<StandardMaterial>,
    pub transportation: Handle<StandardMaterial>,
}

pub fn init_materials(mut assets: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.insert_resource(Materials {
        default: assets.add(c(COLORS.overlay2)),
        residential: assets.add(c(COLORS.rosewater)),
        outbuilding: assets.add(c(COLORS.flamingo)),
        agricultural: assets.add(c(COLORS.green)),
        industrial: assets.add(c(COLORS.teal)),
        commercial: assets.add(c(COLORS.yellow)),
        school: assets.add(c(COLORS.peach)),
        retail: assets.add(c(COLORS.yellow)),
        construction: assets.add(c(COLORS.overlay0)),
        service: assets.add(c(COLORS.blue)),
        hotel: assets.add(c(COLORS.lavender)),
        warehouse: assets.add(c(COLORS.teal)),
        office: assets.add(c(COLORS.overlay1)),
        civic: assets.add(c(COLORS.sky)),
        hospital: assets.add(c(COLORS.red)),
        religious: assets.add(c(COLORS.mauve)),
        transportation: assets.add(c(COLORS.maroon)),
    });
}

fn c(color: Colour) -> StandardMaterial {
    Color::rgba_u8(color.0, color.1, color.2, 255).into()
}
