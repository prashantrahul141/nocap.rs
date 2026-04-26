use ashpd::desktop::{
    PersistMode, Session,
    screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType, Stream},
    screenshot::Screenshot,
};
use nocaprs_platform::{
    Platform, ScreenCaptureContext, ScreenCaptureError, ScreenShot, ScreenShotError,
};
use pipewire::{
    context::ContextRc,
    main_loop::MainLoopRc,
    properties::properties,
    spa::{
        self,
        pod::{Pod, Value, serialize::PodSerializer},
    },
    stream::{StreamFlags, StreamRc},
};
use spa::{
    pod::{ChoiceValue, object, property},
    utils::{Choice, ChoiceEnum, ChoiceFlags, Fraction, Id, Rectangle},
};
use std::io::Cursor;
use tracing::{error, info, instrument, trace, warn};

#[derive(Default, Debug)]
pub struct Linux {
    session: Option<Session<Screencast>>,
    streams: Option<Vec<Stream>>,
}

impl Platform for Linux {
    #[instrument(name = "take_screenshot")]
    async fn take_screenshot(&self) -> Result<ScreenShot, ScreenShotError> {
        info!("attempting to take screenshot");
        let req = Screenshot::request()
            .interactive(true)
            .modal(true)
            .send()
            .await
            .map_err(ScreenShotError::DBus)?;
        trace!("{:?}", req);
        let resp = req.response().map_err(ScreenShotError::DBus)?;
        trace!("{:?}", resp);
        Ok(ScreenShot::Filepath(resp.uri().to_string()))
    }

    #[instrument(name = "create_screencapture_session")]
    async fn create_screencapture_session(
        &mut self,
    ) -> Result<ScreenCaptureContext, ScreenCaptureError> {
        info!("attempting to create screencapture session");

        let proxy = Screencast::new()
            .await
            .inspect_err(|e| error!("failed to create proxy err = {e}"))
            .map_err(ScreenCaptureError::DBus)?;

        let session = proxy
            .create_session(Default::default())
            .await
            .inspect_err(|e| error!("failed to create session err = {e}"))
            .map_err(ScreenCaptureError::DBus)?;

        proxy
            .select_sources(
                &session,
                SelectSourcesOptions::default()
                    .set_cursor_mode(CursorMode::Embedded)
                    .set_sources(SourceType::Monitor | SourceType::Window)
                    .set_multiple(true)
                    .set_persist_mode(PersistMode::DoNot),
            )
            .await
            .inspect_err(|e| error!("failed to select source {e}"))
            .map_err(ScreenCaptureError::DBus)?;
        self.streams = Some(
            proxy
                .start(&session, None, Default::default())
                .await
                .inspect_err(|e| error!("failed to start session err = {e}"))
                .map_err(ScreenCaptureError::DBus)?
                .response()
                .inspect_err(|e| error!("failed to get response from session err = {e}"))
                .map_err(ScreenCaptureError::DBus)?
                .streams()
                .to_vec(),
        );
        self.session = Some(session);

        Ok(ScreenCaptureContext {})
    }

    #[instrument(name = "start_screencapture")]
    async fn start_screencapture(&mut self) -> Result<(), ()> {
        info!("attempting to start screen capture");
        if self.session.is_none() || self.streams.is_none() {
            error!("called start screencapture without initialization, returning");
            return Err(());
        }

        let mainloop = MainLoopRc::new(None).unwrap();
        let context = ContextRc::new(&mainloop, None).unwrap();
        let core = context.connect_rc(None).unwrap();
        let stream = StreamRc::new(
            core,
            "nocaprs",
            properties!(
                *pipewire::keys::MEDIA_TYPE => "Video",
                *pipewire::keys::MEDIA_CATEGORY => "Capture",
                *pipewire::keys::MEDIA_ROLE => "Screen",
            ),
        )
        .unwrap();

        let mut video_info = spa::param::video::VideoInfoRaw::new();
        video_info.set_format(spa::param::video::VideoFormat::BGRx);
        video_info.set_size(spa::utils::Rectangle {
            width: 1920,
            height: 1080,
        });
        video_info.set_framerate(spa::utils::Fraction { num: 30, denom: 1 });

        let video_obj = object! {
            spa::utils::SpaTypes::ObjectParamFormat,
            spa::param::ParamType::EnumFormat,

            property!(
                spa::param::format::FormatProperties::MediaType,
                Id,
                spa::param::format::MediaType::Video
            ),

            property!(
                spa::param::format::FormatProperties::MediaSubtype,
                Id,
                spa::param::format::MediaSubtype::Raw
            ),

            property!(
                spa::param::format::FormatProperties::VideoFormat,
                Choice,
                ChoiceValue::Id(Choice(
                    ChoiceFlags::empty(),
                    ChoiceEnum::Enum {
                        default: Id(spa::param::video::VideoFormat::BGRx.as_raw()),
                        alternatives: vec![
                            Id(spa::param::video::VideoFormat::BGRx.as_raw()),
                            Id(spa::param::video::VideoFormat::RGBx.as_raw()),
                            Id(spa::param::video::VideoFormat::RGBA.as_raw()),
                        ],
                    }
                ))
            ),

            property!(
                spa::param::format::FormatProperties::VideoSize,
                Choice,
                ChoiceValue::Rectangle(Choice(
                    ChoiceFlags::empty(),
                    ChoiceEnum::Range {
                        default: Rectangle { width: 1280, height: 720 },
                        min: Rectangle { width: 1, height: 1 },
                        max: Rectangle { width: 4096, height: 4096 },
                    }
                ))
            ),

            property!(
                spa::param::format::FormatProperties::VideoFramerate,
                Choice,
                ChoiceValue::Fraction(Choice(
                    ChoiceFlags::empty(),
                    ChoiceEnum::Range {
                        default: Fraction { num: 30, denom: 1 },
                        min: Fraction { num: 1, denom: 1 },
                        max: Fraction { num: 144, denom: 1 },
                    }
                ))
            ),
        };
        let values_video: Vec<u8> =
            PodSerializer::serialize(Cursor::new(Vec::new()), &Value::Object(video_obj))
                .unwrap()
                .0
                .into_inner();

        // let mut audio_info = spa::param::audio::AudioInfoRaw::new();
        // audio_info.set_format(spa::param::audio::AudioFormat::F32LE);

        // let audio_obj = pipewire::spa::pod::Object {
        //     type_: pipewire::spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
        //     id: pipewire::spa::param::ParamType::EnumFormat.as_raw(),
        //     properties: audio_info.into(),
        // };
        // let values_audio: Vec<u8> = PodSerializer::serialize(
        //     Cursor::new(Vec::new()),
        //     &pipewire::spa::pod::Value::Object(audio_obj),
        // )
        // .unwrap()
        // .0
        // .into_inner();

        let mut params = [
            // Pod::from_bytes(&values_audio).unwrap(),
            Pod::from_bytes(&values_video).unwrap(),
        ];

        let node = self.streams.as_ref().unwrap()[0].pipe_wire_node_id();

        info!("creating listner");
        let _listener = stream
            .add_local_listener()
            .state_changed(|stream, _userdata, old, new| {
                info!("STATE: {:?} -> {:?}", old, new);
                if new == pipewire::stream::StreamState::Paused {
                    println!("activating stream");
                    stream.set_active(true).unwrap();
                }
            })
            .param_changed(|stream, _, id, param| {
                warn!("PARAM CHANGED");
                if id == spa::param::ParamType::Format.as_raw()
                    && let Some(param) = param
                {
                    warn!("FORMAT RECEIVED");

                    stream.update_params(&mut [param]).unwrap();
                }
            })
            .process(|stream, _: &mut usize| {
                dbg!("\n\n\n saving frame\n\n");
                let mut buffer = stream.dequeue_buffer().unwrap();
                let data = buffer.datas_mut();
                let d = data.get_mut(0).unwrap();
                let frame = d.data().unwrap();
                info!("process frame");
                info!("{}", frame.len());
            });

        info!("connecting to pipewire stream");
        stream
            .connect(
                spa::utils::Direction::Input,
                Some(node),
                StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS,
                &mut params,
            )
            .unwrap();

        info!("starting mainloop");
        mainloop.run();

        info!("dropping listner");
        Ok(())
    }
}
