use std::collections::HashMap;

use tetra::math::Vec2;
use tetra::Context;

use crate::image_assets::ImageAssets;
use crate::sprite::Sprite;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ParticleDrawLayer {
    Bottomest,
    Explosion,
    BulletHit,
    FiringBullet,
    Topest,
}

pub struct ParticleSpawnNode {
    particle_type_number: u128,
    position: Vec2<f32>,
    draw_layer: ParticleDrawLayer,
    extra: String,
}

impl ParticleSpawnNode {
    pub fn new(
        particle_type_number: u128,
        position: Vec2<f32>,
        draw_layer: ParticleDrawLayer,
        extra: &str,
    ) -> ParticleSpawnNode {
        ParticleSpawnNode {
            particle_type_number: particle_type_number,
            position: position,
            draw_layer: draw_layer,
            extra: String::from(extra),
        }
    }
}

/// Handle all particles. (Inactive/ Active)
pub struct ParticleManager {
    pub active_particles_layer: HashMap<ParticleDrawLayer, Vec<Particle>>,
    pub inactive_particles: Vec<Particle>,
    /// Use for keepping list of particle that become inactive and has to remove from active list.
    remove_inactive_particle_list: Vec<usize>,
}

impl ParticleManager {
    pub fn new() -> ParticleManager {
        let mut inactive_particles = Vec::new();
        for _i in 1..300 {
            inactive_particles.push(Particle::new());
        }

        clear_particle_spawn_nodes();

        ParticleManager {
            active_particles_layer: HashMap::new(),
            inactive_particles: inactive_particles,
            remove_inactive_particle_list: Vec::new(),
        }
    }

    pub fn update(&mut self, image_assets: &ImageAssets) {
        update_spawn_nodes_list(
            &mut self.inactive_particles,
            &mut self.active_particles_layer,
            image_assets,
        );

        update_active_particles(
            ParticleDrawLayer::Bottomest,
            &mut self.active_particles_layer,
            &mut self.remove_inactive_particle_list,
            &mut self.inactive_particles,
        );

        update_active_particles(
            ParticleDrawLayer::Explosion,
            &mut self.active_particles_layer,
            &mut self.remove_inactive_particle_list,
            &mut self.inactive_particles,
        );

        update_active_particles(
            ParticleDrawLayer::BulletHit,
            &mut self.active_particles_layer,
            &mut self.remove_inactive_particle_list,
            &mut self.inactive_particles,
        );
        update_active_particles(
            ParticleDrawLayer::FiringBullet,
            &mut self.active_particles_layer,
            &mut self.remove_inactive_particle_list,
            &mut self.inactive_particles,
        );

        update_active_particles(
            ParticleDrawLayer::Topest,
            &mut self.active_particles_layer,
            &mut self.remove_inactive_particle_list,
            &mut self.inactive_particles,
        );
    }

    pub fn draw(
        &mut self,
        draw_layer: ParticleDrawLayer,
        ctx: &mut Context,
        image_assets: &ImageAssets,
    ) {
        match self.active_particles_layer.get_mut(&draw_layer) {
            Some(active_particles) => {
                for particle in active_particles.iter_mut() {
                    particle.draw(ctx, image_assets);
                }
            }
            None => (),
        }
    }
}

pub fn update_active_particles(
    draw_layer: ParticleDrawLayer,
    active_particles_layer: &mut HashMap<ParticleDrawLayer, Vec<Particle>>,
    remove_inactive_particle_list: &mut Vec<usize>,
    inactive_particles: &mut Vec<Particle>,
) {
    match active_particles_layer.get_mut(&draw_layer) {
        Some(active_particles) => {
            let mut index = 0;
            for particle in active_particles.iter_mut() {
                if particle.active == true {
                    particle.update();
                } else {
                    remove_inactive_particle_list.push(index);
                }

                index += 1;
            }

            for index in remove_inactive_particle_list.iter().rev() {
                let inactive_particle = active_particles.remove(*index);
                inactive_particles.push(inactive_particle);
            }

            remove_inactive_particle_list.clear();
        }
        None => (),
    }
}

/// Fetching spawn node in the queue and create particle for active list.
pub fn update_spawn_nodes_list(
    inactive_particles: &mut Vec<Particle>,
    active_particles: &mut HashMap<ParticleDrawLayer, Vec<Particle>>,
    image_assets: &ImageAssets,
) {
    let mut particle_types = crate::PARTICLE_TYPE_BANK.lock().unwrap();

    {
        let mut spawn_node_list = crate::PARTICLE_SPAWN_NODES.lock().unwrap();
        for spawn_node in spawn_node_list.iter() {
            spawn_particle(
                inactive_particles,
                active_particles,
                spawn_node,
                &mut particle_types,
                image_assets,
            );
        }

        spawn_node_list.clear();
    }
}

fn spawn_particle(
    inactive_particles: &mut Vec<Particle>,
    active_particles_layers: &mut HashMap<ParticleDrawLayer, Vec<Particle>>,
    spawn_node: &ParticleSpawnNode,
    particle_types: &mut ParticleTypeBank,
    image_assets: &ImageAssets,
) {
    if inactive_particles.len() > 0 {
        let new_node = inactive_particles.pop();
        if new_node.is_some() {
            if active_particles_layers.contains_key(&spawn_node.draw_layer) == false {
                active_particles_layers.insert(spawn_node.draw_layer.clone(), Vec::new());
            }

            let active_particles = active_particles_layers
                .get_mut(&spawn_node.draw_layer)
                .unwrap();
            match particle_types
                .types
                .get_mut(&spawn_node.particle_type_number)
            {
                Some(particle_type) => {
                    let mut new_node = new_node.unwrap();
                    new_node.reset();

                    new_node.particle_type_number = particle_type.particle_type_id();
                    new_node.position = spawn_node.position;
                    new_node.active = true;
                    new_node.sprite.set_loop(false);

                    new_node.parsing_extra(spawn_node.extra.as_str());

                    match new_node.extra.get("scale") {
                        Some(v) => {
                            let scale = v.parse::<f32>().unwrap_or(2.5);
                            new_node.sprite.scale = Vec2::new(scale, scale);
                        }
                        None => {
                            new_node.sprite.scale = Vec2::new(2.5, 2.5);
                        }
                    }

                    particle_type.init(&mut new_node, image_assets);
                    active_particles.push(new_node);
                }
                None => (),
            };
        }
    }
}

fn clear_particle_spawn_nodes() {
    crate::PARTICLE_SPAWN_NODES.lock().unwrap().clear();
}

pub struct Particle {
    /// Object status
    pub active: bool,

    /// Reference number for ParticleTypeBank
    pub particle_type_number: u128,

    /// Position of particle
    pub position: Vec2<f32>,

    /// Rotation of particle. It should be in radian.
    pub rotation: f32,

    /// Velocity of particle
    pub velocity: Vec2<f32>,

    /// Keep sprite data
    pub sprite: Sprite,

    /// Frame count since particle become active.
    pub frame: i32,

    /// Indicate how long particle stay active. It decrease every frame.
    pub life_time: u128,

    pub face_right: bool,

    pub extra: HashMap<String, String>,
}

impl Particle {
    pub fn new() -> Particle {
        Particle {
            active: false,
            particle_type_number: 0,
            position: Vec2::zero(),
            velocity: Vec2::zero(),
            rotation: 0.0,
            sprite: Sprite::new(),
            frame: 0,
            life_time: 0,
            face_right: true,
            extra: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.life_time = 0;
        self.particle_type_number = 0;
        self.position = Vec2::zero();
        self.velocity = Vec2::zero();
        self.rotation = 0.0;
        self.face_right = true;
        self.sprite.reset();
    }

    pub fn update(&mut self) {
        let particle_types = crate::PARTICLE_TYPE_BANK.lock().unwrap();
        match particle_types.get(self.particle_type_number) {
            Some(particle_type) => {
                particle_type.update(self);
            }
            None => (),
        };
    }

    pub fn draw(&mut self, ctx: &mut Context, image_assets: &ImageAssets) {
        let particle_types = crate::PARTICLE_TYPE_BANK.lock().unwrap();
        match particle_types.get(self.particle_type_number) {
            Some(particle_type) => {
                particle_type.draw(ctx, self, image_assets);
            }
            None => (),
        };
    }

    pub fn parsing_extra(&mut self, raw_extra: &str) {
        if raw_extra.len() == 0 {
            return;
        }

        let split: Vec<&str> = raw_extra.split('|').collect();

        for text in split.iter() {
            if text.len() == 0 {
                continue;
            }

            let parameter: Vec<&str> = text.split('=').collect();
            if parameter.len() == 2 {
                self.extra
                    .insert(String::from(parameter[0]), String::from(parameter[1]));
            } else {
                panic!("Incorrect parameter format: {} ({})", text, raw_extra);
            }
        }
    }
}

pub trait ParticleType {
    fn particle_type_id(&self) -> u128;
    fn init(&mut self, particle: &mut Particle, image_assets: &ImageAssets);
    fn update(&self, particle: &mut Particle);
    fn draw(&self, ctx: &mut Context, particle: &mut Particle, image_assets: &ImageAssets);
}

/// Keep all particle types that use in current scene.
pub struct ParticleTypeBank {
    pub types: HashMap<u128, Box<dyn ParticleType + Send + Sync>>,
}

impl ParticleTypeBank {
    pub fn new() -> ParticleTypeBank {
        ParticleTypeBank {
            types: HashMap::new(),
        }
    }

    pub fn get(&self, number: u128) -> Option<&Box<dyn ParticleType + Send + Sync>> {
        self.types.get(&number)
    }

    pub fn add(&mut self, number: u128, particle_type: Box<dyn ParticleType + Send + Sync>) {
        self.types.insert(number, particle_type);
    }

    pub fn setup(&mut self, image_assets: &ImageAssets) {
        let particle_type =
            crate::gameplay::particle_types::explosion::ExplosionParticleType::new(image_assets);
        self.add(particle_type.particle_type_id(), Box::new(particle_type));
    }

    pub fn clear(&mut self) {
        self.types.clear();
    }
}
