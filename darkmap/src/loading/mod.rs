use std::marker::{PhantomData, Send};

use bevy::{
    prelude::*,
    tasks::{block_on, AsyncComputeTaskPool, Task},
};
use futures::{future::poll_immediate, Future};
use geo::{HaversineDestination, Point};

#[derive(Default)]
pub struct LoadingPlugin<T: LoadType> {
    _marker: PhantomData<T>,
}

impl<T: LoadType> LoadingPlugin<T> {
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T: LoadType> Plugin for LoadingPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (start_loading::<T>, finish_loading::<T>));
    }
}

#[derive(Component)]
pub struct LoadRequest {
    center: Point,
    radius: f64,
    // _marker: PhantomData<T>,
}

impl LoadRequest {
    pub fn new(center: Point, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn bbox(&self) -> String {
        let north = self.center.haversine_destination(0., self.radius);
        let east = self.center.haversine_destination(90., self.radius);
        let south = self.center.haversine_destination(180., self.radius);
        let west = self.center.haversine_destination(270., self.radius);

        format!("{},{},{},{}", south.y(), west.x(), north.y(), east.x())
    }
}

impl Clone for LoadRequest {
    fn clone(&self) -> Self {
        Self {
            center: self.center,
            radius: self.radius,
        }
    }
}

#[derive(Component)]
pub struct LoadTask<T: LoadType> {
    task: Task<anyhow::Result<Vec<T::Bundle>>>,
}

fn start_loading<T: LoadType>(
    query: Query<(Entity, &LoadRequest), Without<LoadTask<T>>>,
    mut commands: Commands,
) {
    for (entity, req) in &mut query.iter() {
        let req = req.clone();
        let task = AsyncComputeTaskPool::get().spawn(async move { T::load(req).await });

        commands
            .entity(entity)
            .insert(LoadTask::<T> { task })
            .remove::<LoadRequest>();
    }
}

fn finish_loading<T: LoadType>(
    mut query: Query<(Entity, &mut LoadTask<T>)>,
    mut commands: Commands,
) {
    for (entity, mut task) in &mut query.iter_mut() {
        if let Some(result) = block_on(poll_immediate(&mut task.task)) {
            commands.entity(entity).remove::<LoadTask<T>>();

            match result {
                Ok(bundles) => {
                    for bundle in bundles {
                        commands.spawn(bundle);
                    }
                }
                Err(e) => {
                    error!("Failed to load: {}", e);
                }
            }
        }
    }
}

pub trait LoadType: Sized + Send + Sync + 'static {
    type Bundle: Bundle;
    fn load(req: LoadRequest) -> impl Future<Output = anyhow::Result<Vec<Self::Bundle>>> + Send;
}
