use rand::{thread_rng, Rng};
use strum::{EnumIter, FromRepr, IntoEnumIterator};

mod dht22;
mod esp32;

slint::include_modules!();

#[repr(i32)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, FromRepr, EnumIter)]
enum Dice {
    D4 = 4,
    D6 = 6,
    D8 = 8,
    D10 = 10,
    D12 = 12,
    #[default]
    D20 = 20,
    D100 = 100,
}

impl Dice {
    fn dec(&self) -> Option<Self> {
        Dice::iter().rev().skip_while(|d| d != self).skip(1).next()
    }

    fn inc(&self) -> Option<Self> {
        Dice::iter().skip_while(|d| d != self).skip(1).next()
    }

    fn roll(&self, quantity: i32) -> Vec<i32> {
        let mut rng = thread_rng();
        (0..quantity)
            .map(|_| rng.gen_range(1..(*self as i32)))
            .collect()
    }
}

/// Our App struct that holds the UI
struct App {
    ui: MainWindow,
}

impl App {
    /// Create a new App struct.
    ///
    /// The App struct initializes the UI and the weather controller.
    fn new() -> anyhow::Result<Self> {
        // Make a new MainWindow
        let ui = MainWindow::new().map_err(|e| anyhow::anyhow!(e))?;

        // init default
        let model = ViewModel::get(&ui);
        model.set_current(DiceRoll {
            dice: Dice::default() as i32,
            quantity: 1,
        });
        model.set_roll_result("Ready to roll".into());

        ui.on_decrease_quantity(|quantity| {
            log::error!("decrease quantity {quantity}");
            (quantity - 1).max(0)
        });

        ui.on_increase_quantity(|quantity| {
            log::error!("increase quantity {quantity}");
            (quantity + 1).min(100)
        });

        ui.on_decrease_dice(|dice| {
            log::error!("decrease dice {dice}");
            if let Some(dec) = Dice::from_repr(dice).unwrap_or_default().dec() {
                dec as i32
            } else {
                dice
            }
        });

        ui.on_increase_dice(|dice| {
            log::error!("increase dice {dice}");
            if let Some(inc) = Dice::from_repr(dice).unwrap_or_default().inc() {
                inc as i32
            } else {
                dice
            }
        });

        ui.on_roll({
            let ui_handle = ui.as_weak();
            move |quantity, dice| {
                let ui = ui_handle.unwrap();
                let model = ViewModel::get(&ui);
                let dice = Dice::from_repr(dice).unwrap_or_default();
                let roll = dice.roll(quantity);
                model.set_current(DiceRoll {
                    dice: dice as i32,
                    quantity,
                });
                if roll.len() > 1 {
                    model.set_roll_explanation(
                        roll.iter()
                            .fold(String::new(), |mut acc, dice| {
                                if !acc.is_empty() {
                                    acc.push_str(" + ");
                                }
                                acc.push_str(&dice.to_string());
                                acc
                            })
                            .into(),
                    );
                } else {
                    model.set_roll_explanation("".into());
                }
                model.set_roll_result(roll.into_iter().sum::<i32>().to_string().into());
            }
        });

        // Return the App struct
        Ok(Self { ui })
    }

    /// Run the App
    fn run(&mut self) -> anyhow::Result<()> {
        // Run the UI (and map an error to an anyhow::Error).
        self.ui.run().map_err(|e| anyhow::anyhow!(e))
    }
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Set the platform
    slint::platform::set_platform(esp32::EspPlatform::new()).unwrap();

    let mut app = App::new()?;

    app.run()
}
