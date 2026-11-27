use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk};
use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

struct Lobe {
    radius_x: f32,
    radius_y: f32,
    drift_x: f32,
    drift_y: f32,
    freq_x: f64,
    freq_y: f64,
    freq_scale: f64,
    freq_angle: f64,
    base_angle: f32,
    angle_span: f32,
    peak_alpha: f32,
    phase: f64,
}

const LOBES: [Lobe; 4] = [
    Lobe {
        radius_x: 195.0,
        radius_y: 125.0,
        drift_x: 26.0,
        drift_y: 18.0,
        freq_x: 0.41,
        freq_y: 0.59,
        freq_scale: 0.33,
        freq_angle: 0.23,
        base_angle: -18.0,
        angle_span: 24.0,
        peak_alpha: 0.22,
        phase: 0.0,
    },
    Lobe {
        radius_x: 130.0,
        radius_y: 185.0,
        drift_x: 22.0,
        drift_y: 24.0,
        freq_x: 0.67,
        freq_y: 0.37,
        freq_scale: 0.47,
        freq_angle: 0.29,
        base_angle: 32.0,
        angle_span: 28.0,
        peak_alpha: 0.18,
        phase: 1.7,
    },
    Lobe {
        radius_x: 220.0,
        radius_y: 105.0,
        drift_x: 32.0,
        drift_y: 20.0,
        freq_x: 0.53,
        freq_y: 0.73,
        freq_scale: 0.29,
        freq_angle: 0.19,
        base_angle: 64.0,
        angle_span: 32.0,
        peak_alpha: 0.14,
        phase: 3.1,
    },
    Lobe {
        radius_x: 150.0,
        radius_y: 110.0,
        drift_x: 18.0,
        drift_y: 26.0,
        freq_x: 0.83,
        freq_y: 0.47,
        freq_scale: 0.61,
        freq_angle: 0.31,
        base_angle: -52.0,
        angle_span: 22.0,
        peak_alpha: 0.16,
        phase: 4.9,
    },
];

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct HeroGlow {
        pub target: glib::WeakRef<gtk::Widget>,
        pub seed: Cell<f64>,
        pub start_us: Cell<Option<i64>>,
        pub elapsed: Cell<f64>,
        pub ticking: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HeroGlow {
        const NAME: &'static str = "RovarchHeroGlow";
        type Type = super::HeroGlow;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for HeroGlow {}

    impl WidgetImpl for HeroGlow {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let Some(target) = self.target.upgrade() else {
                return;
            };
            let Some(bounds) = target.compute_bounds(&*widget) else {
                return;
            };

            let cx = bounds.x() + bounds.width() * 0.5;
            let cy = bounds.y() + bounds.height() * 0.5;
            let t = self.elapsed.get() + self.seed.get();

            for lobe in &LOBES {
                let p = t + lobe.phase;
                let dx = (p * lobe.freq_x).sin() as f32 * lobe.drift_x
                    + (p * lobe.freq_x * 1.618).cos() as f32 * (lobe.drift_x * 0.4);
                let dy = (p * lobe.freq_y).cos() as f32 * lobe.drift_y
                    + (p * lobe.freq_y * 1.414).sin() as f32 * (lobe.drift_y * 0.4);
                let scale = 1.0 + 0.16 * (p * lobe.freq_scale).sin() as f32;
                let angle = lobe.base_angle + lobe.angle_span * (p * lobe.freq_angle).sin() as f32;
                let alpha =
                    lobe.peak_alpha * (0.82 + 0.18 * (p * (lobe.freq_scale * 1.37)).cos() as f32);

                let rx = lobe.radius_x * scale;
                let ry = lobe.radius_y * scale;
                let rect = graphene::Rect::new(-rx, -ry, rx * 2.0, ry * 2.0);
                let center = graphene::Point::new(0.0, 0.0);

                let stops = [
                    gsk::ColorStop::new(0.0, gdk::RGBA::new(0.937, 0.267, 0.267, alpha)),
                    gsk::ColorStop::new(0.38, gdk::RGBA::new(0.937, 0.267, 0.267, alpha * 0.48)),
                    gsk::ColorStop::new(0.72, gdk::RGBA::new(0.863, 0.149, 0.149, alpha * 0.12)),
                    gsk::ColorStop::new(1.0, gdk::RGBA::new(0.863, 0.149, 0.149, 0.0)),
                ];

                snapshot.save();
                snapshot.translate(&graphene::Point::new(cx + dx, cy + dy));
                snapshot.rotate(angle);
                snapshot.append_radial_gradient(&rect, &center, rx, ry, 0.0, 1.0, &stops);
                snapshot.restore();
            }
        }
    }
}

glib::wrapper! {
    pub struct HeroGlow(ObjectSubclass<imp::HeroGlow>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

pub fn hero_glow(target: &impl IsA<gtk::Widget>) -> HeroGlow {
    let glow: HeroGlow = glib::Object::builder()
        .property("hexpand", true)
        .property("vexpand", true)
        .property("can-target", false)
        .build();

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as f64 / 1_000_000.0)
        .unwrap_or(0.0);

    let imp = glow.imp();
    imp.target.set(Some(target.as_ref()));
    imp.seed.set((nanos % 100.0) + 1.0);

    glow.connect_map(|glow| {
        let imp = glow.imp();
        if imp.ticking.get() {
            return;
        }
        imp.ticking.set(true);
        glow.add_tick_callback(|glow, clock| {
            if !glow.is_mapped() {
                glow.imp().ticking.set(false);
                glow.imp().start_us.set(None);
                return glib::ControlFlow::Break;
            }
            let now = clock.frame_time();
            let imp = glow.imp();
            let start = match imp.start_us.get() {
                Some(start) => start,
                None => {
                    imp.start_us.set(Some(now));
                    now
                }
            };
            imp.elapsed.set((now - start) as f64 / 1_000_000.0);
            glow.queue_draw();
            glib::ControlFlow::Continue
        });
    });

    glow
}
