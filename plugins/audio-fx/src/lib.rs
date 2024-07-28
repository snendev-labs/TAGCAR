use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
};
use bevy_kira_audio::{
    prelude::{AudioChannel, AudioSource},
    AudioApp, AudioControl,
};

pub struct AudioFxPlugin;

impl Plugin for AudioFxPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AudioFxAssetsState>()
            .add_loading_state(
                LoadingState::new(AudioFxAssetsState::Loading)
                    .load_collection::<AudioFxAssets>()
                    .continue_to_state(AudioFxAssetsState::Loaded),
            )
            .add_audio_channel::<CrashFxChannel>()
            .add_audio_channel::<ScoreFxChannel>()
            .add_systems(
                Update,
                (
                    Self::play_crash_fx.in_set(AudioFxSystems::CrashFx),
                    Self::play_lap_fx.in_set(AudioFxSystems::ScoreFx),
                )
                    .run_if(in_state(AudioFxAssetsState::Loaded)),
            );
    }
}

impl AudioFxPlugin {
    fn play_crash_fx(channel: Res<AudioChannel<CrashFxChannel>>, music_assets: Res<AudioFxAssets>) {
        channel.play(music_assets.crash_fx.clone()).with_volume(0.5);
    }

    fn play_lap_fx(channel: Res<AudioChannel<ScoreFxChannel>>, music_assets: Res<AudioFxAssets>) {
        channel.play(music_assets.score_fx.clone()).with_volume(0.5);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub enum AudioFxSystems {
    CrashFx,
    ScoreFx,
}

#[derive(Clone)]
#[derive(Resource)]
pub struct CrashFxChannel;

#[derive(Clone)]
#[derive(Resource)]
pub struct ScoreFxChannel;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[derive(States)]
pub enum AudioFxAssetsState {
    #[default]
    Loading,
    Loaded,
}

#[derive(AssetCollection, Resource)]
pub struct AudioFxAssets {
    #[cfg_attr(not(target_arch = "wasm32"), asset(path = "audio/crash-fx.wav"))]
    #[cfg_attr(target_arch = "wasm32", asset(path = "audio/crash-fx.mp3"))]
    pub crash_fx: Handle<AudioSource>,
    #[cfg_attr(not(target_arch = "wasm32"), asset(path = "audio/score-fx.wav"))]
    #[cfg_attr(target_arch = "wasm32", asset(path = "audio/score-fx.mp3"))]
    pub score_fx: Handle<AudioSource>,
}
