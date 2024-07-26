use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyComponent;

pub type Entropy = EntropyComponent<WyRand>;
