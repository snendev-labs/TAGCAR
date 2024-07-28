use bevy::prelude::*;
use bevy_rand::prelude::EntropyPlugin as RandEntropyPlugin;

pub use bevy_prng::WyRand;
pub use bevy_rand::prelude::*;
pub use rand_core::RngCore;

pub struct EntropyPlugin;

impl Plugin for EntropyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RandEntropyPlugin::<WyRand>::default());
    }
}

pub type Entropy = EntropyComponent<WyRand>;

#[derive(Clone, Debug, Default)]
#[derive(Component, Reflect)]
pub struct EntropyBundle {
    entropy: Entropy,
}

impl EntropyBundle {
    pub fn new(global: &mut GlobalEntropy<WyRand>) -> Self {
        Self {
            entropy: global.fork_rng(),
        }
    }
}
