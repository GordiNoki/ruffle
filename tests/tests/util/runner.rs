use crate::RUN_IMG_TESTS;
use anyhow::{anyhow, Result};
use regex::Regex;
use ruffle_core::backend::log::LogBackend;
use ruffle_core::backend::navigator::{NullExecutor, NullNavigatorBackend};
use ruffle_core::events::MouseButton as RuffleMouseButton;
use ruffle_core::limits::ExecutionLimit;
use ruffle_core::tag_utils::SwfMovie;
use ruffle_core::{Player, PlayerBuilder, PlayerEvent};
use ruffle_input_format::{AutomatedEvent, InputInjector, MouseButton as InputMouseButton};
#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::backend::WgpuRenderBackend;
#[cfg(feature = "imgtests")]
use ruffle_render_wgpu::{target::TextureTarget, wgpu};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct TestLogBackend {
    trace_output: Rc<RefCell<Vec<String>>>,
}

impl TestLogBackend {
    pub fn new(trace_output: Rc<RefCell<Vec<String>>>) -> Self {
        Self { trace_output }
    }
}

impl LogBackend for TestLogBackend {
    fn avm_trace(&self, message: &str) {
        self.trace_output.borrow_mut().push(message.to_string());
    }
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
pub fn run_swf(
    swf_path: &Path,
    num_frames: u32,
    before_start: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
    mut injector: InputInjector,
    before_end: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
    #[allow(unused)] mut check_img: bool,
    frame_time_sleep: bool,
) -> Result<String> {
    #[allow(unused_assignments)]
    {
        check_img &= RUN_IMG_TESTS;
    }

    let base_path = Path::new(swf_path).parent().unwrap();
    let mut executor = NullExecutor::new();
    let movie = SwfMovie::from_path(swf_path, None).map_err(|e| anyhow!(e.to_string()))?;
    let frame_time = 1000.0 / movie.frame_rate().to_f64();
    let frame_time_duration = Duration::from_millis(frame_time as u64);
    let trace_output = Rc::new(RefCell::new(Vec::new()));

    #[allow(unused_mut)]
    let mut builder = PlayerBuilder::new();

    #[cfg(feature = "imgtests")]
    if check_img {
        const BACKEND: wgpu::Backends = wgpu::Backends::PRIMARY;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: BACKEND,
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
        });

        let descriptors =
            futures::executor::block_on(WgpuRenderBackend::<TextureTarget>::build_descriptors(
                BACKEND,
                instance,
                None,
                Default::default(),
                None,
            ))?;

        let width = movie.width().to_pixels() as u32;
        let height = movie.height().to_pixels() as u32;

        let target = TextureTarget::new(&descriptors.device, (width, height))?;

        builder = builder
            .with_renderer(WgpuRenderBackend::new(Arc::new(descriptors), target, 4)?)
            .with_viewport_dimensions(width, height, 1.0);
    };

    let player = builder
        .with_log(TestLogBackend::new(trace_output.clone()))
        .with_navigator(NullNavigatorBackend::with_base_path(base_path, &executor)?)
        .with_max_execution_duration(Duration::from_secs(300))
        .with_viewport_dimensions(
            movie.width().to_pixels() as u32,
            movie.height().to_pixels() as u32,
            1.0,
        )
        .with_movie(movie)
        .build();

    before_start(player.clone())?;

    for _ in 0..num_frames {
        // If requested, ensure that the 'expected' amount of
        // time actually elapses between frames. This is useful for
        // tests that call 'flash.utils.getTimer()' and use
        // 'setInterval'/'flash.utils.Timer'
        //
        // Note that when Ruffle actually runs frames, we can
        // execute frames faster than this in order to 'catch up'
        // if we've fallen behind. However, in order to make regression
        // tests deterministic, we always call 'update_timers' with
        // an elapsed time of 'frame_time'. By sleeping for 'frame_time_duration',
        // we ensure that the result of 'flash.utils.getTimer()' is consistent
        // with timer execution (timers will see an elapsed time of *at least*
        // the requested timer interval).
        if frame_time_sleep {
            std::thread::sleep(frame_time_duration);
        }

        while !player
            .lock()
            .unwrap()
            .preload(&mut ExecutionLimit::exhausted())
        {}

        player.lock().unwrap().run_frame();
        player.lock().unwrap().update_timers(frame_time);
        executor.run();

        injector.next(|evt, _btns_down| {
            player.lock().unwrap().handle_event(match evt {
                AutomatedEvent::MouseDown { pos, btn } => PlayerEvent::MouseDown {
                    x: pos.0,
                    y: pos.1,
                    button: match btn {
                        InputMouseButton::Left => RuffleMouseButton::Left,
                        InputMouseButton::Middle => RuffleMouseButton::Middle,
                        InputMouseButton::Right => RuffleMouseButton::Right,
                    },
                },
                AutomatedEvent::MouseMove { pos } => PlayerEvent::MouseMove { x: pos.0, y: pos.1 },
                AutomatedEvent::MouseUp { pos, btn } => PlayerEvent::MouseUp {
                    x: pos.0,
                    y: pos.1,
                    button: match btn {
                        InputMouseButton::Left => RuffleMouseButton::Left,
                        InputMouseButton::Middle => RuffleMouseButton::Middle,
                        InputMouseButton::Right => RuffleMouseButton::Right,
                    },
                },
                AutomatedEvent::Wait => unreachable!(),
            });
        });
        // Rendering has side-effects (such as processing 'DisplayObject.scrollRect' updates)
        player.lock().unwrap().render();
    }

    // Render the image to disk
    // FIXME: Determine how we want to compare against on on-disk image
    #[cfg(feature = "imgtests")]
    if check_img {
        let mut player_lock = player.lock().unwrap();
        player_lock.render();
        let renderer = player_lock
            .renderer_mut()
            .downcast_mut::<WgpuRenderBackend<TextureTarget>>()
            .unwrap();

        // Use straight alpha, since we want to save this as a PNG
        let actual_image = renderer
            .capture_frame(false)
            .expect("Failed to capture image");

        let info = renderer.descriptors().adapter.get_info();
        let suffix = format!("{}-{:?}", std::env::consts::OS, info.backend);

        let expected_image_path = base_path.join(format!("expected-{}.png", &suffix));
        let expected_image = image::open(&expected_image_path);

        let matches = match expected_image {
            Ok(img) => {
                img.as_rgba8().expect("Expected 8-bit RGBA image").as_raw() == actual_image.as_raw()
            }
            Err(e) => {
                eprintln!(
                    "Failed to open expected image {:?}: {e:?}",
                    &expected_image_path
                );
                false
            }
        };

        if !matches {
            let actual_image_path = base_path.join(format!("actual-{suffix}.png"));
            actual_image.save_with_format(&actual_image_path, image::ImageFormat::Png)?;
            panic!("Test output does not match expected image - saved actual image to {actual_image_path:?}");
        }
    }

    before_end(player)?;

    executor.run();

    let trace = trace_output.borrow().join("\n");
    Ok(trace)
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
/// If a line has a floating point value, it will be compared approxinmately using the given epsilon.
pub fn test_swf_approx(
    swf_path: &Path,
    num_frames: u32,
    simulated_input_path: &Path,
    expected_output_path: &Path,
    num_patterns: &[Regex],
    check_img: bool,
    approx_assert_fn: impl Fn(f64, f64),
) -> Result<()> {
    let injector =
        InputInjector::from_file(simulated_input_path).unwrap_or_else(|_| InputInjector::empty());
    let trace_log = run_swf(
        swf_path,
        num_frames,
        |_| Ok(()),
        injector,
        |_| Ok(()),
        check_img,
        false,
    )?;
    let mut expected_data = std::fs::read_to_string(expected_output_path)?;

    // Strip a trailing newline if it has one.
    if expected_data.ends_with('\n') {
        expected_data = expected_data[0..expected_data.len() - "\n".len()].to_string();
    }

    std::assert_eq!(
        trace_log.lines().count(),
        expected_data.lines().count(),
        "# of lines of output didn't match"
    );

    for (actual, expected) in trace_log.lines().zip(expected_data.lines()) {
        // If these are numbers, compare using approx_eq.
        if let (Ok(actual), Ok(expected)) = (actual.parse::<f64>(), expected.parse::<f64>()) {
            // NaNs should be able to pass in an approx test.
            if actual.is_nan() && expected.is_nan() {
                continue;
            }

            // TODO: Lower this epsilon as the accuracy of the properties improves.
            // if let Some(relative_epsilon) = relative_epsilon {
            //     assert_relative_eq!(
            //         actual,
            //         expected,
            //         epsilon = absolute_epsilon,
            //         max_relative = relative_epsilon
            //     );
            // } else {
            //     assert_abs_diff_eq!(actual, expected, epsilon = absolute_epsilon);
            // }
            approx_assert_fn(actual, expected);
        } else {
            let mut found = false;
            // Check each of the user-provided regexes for a match
            for pattern in num_patterns {
                if let (Some(actual_captures), Some(expected_captures)) =
                    (pattern.captures(actual), pattern.captures(expected))
                {
                    found = true;
                    std::assert_eq!(
                        actual_captures.len(),
                        expected_captures.len(),
                        "Differing numbers of regex captures"
                    );

                    // Each capture group (other than group 0, which is always the entire regex
                    // match) represents a floating-point value
                    for (actual_val, expected_val) in actual_captures
                        .iter()
                        .skip(1)
                        .zip(expected_captures.iter().skip(1))
                    {
                        let actual_num = actual_val
                            .expect("Missing capture gruop value for 'actual'")
                            .as_str()
                            .parse::<f64>()
                            .expect("Failed to parse 'actual' capture group as float");
                        let expected_num = expected_val
                            .expect("Missing capture gruop value for 'expected'")
                            .as_str()
                            .parse::<f64>()
                            .expect("Failed to parse 'expected' capture group as float");
                        approx_assert_fn(actual_num, expected_num);
                    }
                    let modified_actual = pattern.replace(actual, "");
                    let modified_expected = pattern.replace(expected, "");
                    assert_eq!(modified_actual, modified_expected);
                    break;
                }
            }
            if !found {
                assert_eq!(actual, expected);
            }
        }
    }
    Ok(())
}

/// Loads an SWF and runs it through the Ruffle core for a number of frames.
/// Tests that the trace output matches the given expected output.
#[allow(clippy::too_many_arguments)]
pub fn test_swf_with_hooks(
    swf_path: &Path,
    num_frames: u32,
    simulated_input_path: &Path,
    expected_output_path: &Path,
    before_start: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
    before_end: impl FnOnce(Arc<Mutex<Player>>) -> Result<()>,
    check_img: bool,
    frame_time_sleep: bool,
) -> Result<()> {
    let injector =
        InputInjector::from_file(simulated_input_path).unwrap_or_else(|_| InputInjector::empty());
    let mut expected_output = std::fs::read_to_string(expected_output_path)?.replace("\r\n", "\n");

    // Strip a trailing newline if it has one.
    if expected_output.ends_with('\n') {
        expected_output = expected_output[0..expected_output.len() - "\n".len()].to_string();
    }

    let trace_log = run_swf(
        swf_path,
        num_frames,
        before_start,
        injector,
        before_end,
        check_img,
        frame_time_sleep,
    )?;
    assert_eq!(
        trace_log, expected_output,
        "ruffle output != flash player output"
    );

    Ok(())
}
