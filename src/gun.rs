use bevy::prelude::*;
use cgmath::{self, Angle, Rad};
use rand::{self, Rng};

pub trait Gun: Send + Sync {
    fn new() -> Box<Self>
    where
        Self: Sized;

    fn shoot(
        &mut self,
        time: Res<Time>,
        mouse: Res<Input<MouseButton>>,
        player_transform: &Transform,
        angle: Rad<f32>,
        materials: Res<crate::Materials>,
        commands: Commands,
    );
}

pub struct Shotgun {
    last_shot: f64,
}

impl Gun for Shotgun {
    fn new() -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(Shotgun {
            last_shot: 0.0
        })
    }

    fn shoot(
        &mut self,
        time: Res<Time>,
        mouse: Res<Input<MouseButton>>,
        player_transform: &Transform,
        angle: Rad<f32>,
        materials: Res<crate::Materials>,
        mut commands: Commands,
    ) {
        if (time.seconds_since_startup() - self.last_shot) > 0.2 {
            if mouse.just_pressed(MouseButton::Left) {
                let mut random = rand::thread_rng();
                for _index in 0..5 {
                    let mut transform = player_transform.clone();
                    transform.translation.z = 0.0;

                    commands
                        .spawn()
                        .insert_bundle(SpriteBundle {
                            sprite: Sprite::new(Vec2::new(10.0, 4.0)),
                            material: materials.bullet.clone(),
                            transform,
                            ..Default::default()
                        })
                        .insert(crate::Bullet {})
                        .insert(crate::Velocity {
                            x: (angle.cos() + random.gen_range(-0.1..=0.1)) * 6.0,
                            y: (angle.sin() + random.gen_range(-0.1..=0.1)) * 6.0,
                        });
                }
            }
        }
    }
}

pub struct Pistol {
    last_shot: f64,
}

impl Gun for Pistol {
    fn new() -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(Pistol { last_shot: 0.0 })
    }

    fn shoot(
        &mut self,
        time: Res<Time>,
        mouse: Res<Input<MouseButton>>,
        player_transform: &Transform,
        angle: Rad<f32>,
        materials: Res<crate::Materials>,
        mut commands: Commands,
    ) {
        if (time.seconds_since_startup() - self.last_shot) > 0.1 {
            if mouse.pressed(MouseButton::Left) {
                let velocity = crate::Velocity {
                    x: angle.cos() * 6.0,
                    y: angle.sin() * 6.0,
                };

                let mut transform = player_transform.clone();
                transform.translation.z = 0.0;
                commands
                    .spawn()
                    .insert_bundle(SpriteBundle {
                        sprite: Sprite::new(Vec2::new(10.0, 4.0)),
                        material: materials.bullet.clone(),
                        transform,
                        ..Default::default()
                    })
                    .insert(crate::Bullet {})
                    .insert(velocity);

                self.last_shot = time.seconds_since_startup();
            }
        }
    }
}
