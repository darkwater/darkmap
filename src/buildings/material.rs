use bevy::prelude::*;

use crate::{color, COLORS};

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
        default: assets.add(color(COLORS.overlay2).into()),
        residential: assets.add(color(COLORS.rosewater).into()),
        outbuilding: assets.add(color(COLORS.flamingo).into()),
        agricultural: assets.add(color(COLORS.green).into()),
        industrial: assets.add(color(COLORS.teal).into()),
        commercial: assets.add(color(COLORS.yellow).into()),
        school: assets.add(color(COLORS.peach).into()),
        retail: assets.add(color(COLORS.yellow).into()),
        construction: assets.add(color(COLORS.overlay0).into()),
        service: assets.add(color(COLORS.blue).into()),
        hotel: assets.add(color(COLORS.lavender).into()),
        warehouse: assets.add(color(COLORS.teal).into()),
        office: assets.add(color(COLORS.overlay1).into()),
        civic: assets.add(color(COLORS.sky).into()),
        hospital: assets.add(color(COLORS.red).into()),
        religious: assets.add(color(COLORS.mauve).into()),
        transportation: assets.add(color(COLORS.maroon).into()),
    });
}
