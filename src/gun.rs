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

    fn reload(&mut self);
}

pub struct Shotgun {
    time_left:  f32,
    mag_size:   u16
}

pub struct Pistol {
    time_left:  f32,
    mag_size:   u16
}

impl Gun for Shotgun {
    fn new() -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(Shotgun {
            time_left:  0.2,
            mag_size:   2
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
        self.time_left -= time.delta_seconds();
        if self.time_left <= 0.0 {
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

                self.mag_size -= 1;

                // Set time before next possible shot
                if self.mag_size == 0 {
                    self.reload();
                } else {
                    self.time_left = 0.5;
                }
            }
        }
    }

    fn reload(&mut self) {
        self.time_left =    1.0;
        self.mag_size =     2;
    }
}


impl Gun for Pistol {
    fn new() -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(Pistol { 
            time_left:  0.0,
            mag_size:   7
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
        self.time_left -= time.delta_seconds();
        if self.time_left <= 0.0 {
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
                
                self.mag_size -= 1;

                // Set time before next possible shot
                if self.mag_size == 0 {
                    self.reload();
                } else {
                    self.time_left = 0.2;
                }


            }
        }
    }

    fn reload(&mut self) {
        self.time_left =    0.8;
        self.mag_size =     7;
    }
}
