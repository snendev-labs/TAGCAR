use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
};
use bevy_kira_audio::{prelude::AudioSource, AudioApp, AudioChannel, AudioControl};

pub struct BgMusicPlugin;

impl Plugin for BgMusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<BgMusicAssetsState>()
            .add_loading_state(
                LoadingState::new(BgMusicAssetsState::Loading)
                    .load_collection::<BgMusicAssets>()
                    .continue_to_state(BgMusicAssetsState::Loaded),
            )
            .add_audio_channel::<BgMusicChannel>()
            .add_systems(
                OnEnter(BgMusicAssetsState::Loaded),
                // after we play once, just keep the loop going forever
                Self::play_bg_music.run_if(run_once()),
            );
    }
}

impl BgMusicPlugin {
    fn play_bg_music(channel: Res<AudioChannel<BgMusicChannel>>, music_assets: Res<BgMusicAssets>) {
        channel
            .play(music_assets.bg_track.clone())
            .looped()
            .with_volume(0.5);
    }
}

#[derive(Clone)]
#[derive(Resource)]
pub struct BgMusicChannel;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[derive(States)]
pub enum BgMusicAssetsState {
    #[default]
    Loading,
    Loaded,
}

#[derive(AssetCollection, Resource)]
pub struct BgMusicAssets {
    #[cfg_attr(not(target_arch = "wasm32"), asset(path = "audio/bg-music.wav"))]
    #[cfg_attr(target_arch = "wasm32", asset(path = "audio/bg-music.mp3"))]
    pub bg_track: Handle<AudioSource>,
}
