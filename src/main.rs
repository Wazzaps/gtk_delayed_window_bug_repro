use adw::prelude::{ApplicationExt, ApplicationExtManual};
use glib::{Continue, MainContext, PRIORITY_HIGH};
use gtk::prelude::{BoxExt, WidgetExt};
use std::cell::RefCell;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = MainContext::channel(PRIORITY_HIGH);

    std::thread::spawn(move || {
        // If "wait" is the first arg
        if std::env::args().nth(1) == Some("wait".to_string()) {
            eprintln!("Waiting 1 sec before spawning window, this will trigger the bug...");
            // Wait for main loop to settle
            sleep(Duration::from_secs(1));
        }

        // Trigger window creation
        eprintln!("Sending event on tid={:?}", std::thread::current().id());
        sender.send(()).unwrap();
    });

    main_loop(receiver);

    Ok(())
}

fn main_loop(event_recv: glib::Receiver<()>) {
    let windows: RefCell<Vec<adw::Window>> = RefCell::new(Vec::new());

    let application = adw::Application::builder()
        .application_id("com.example.GtkBugRepro")
        // .flags(gio::ApplicationFlags::IS_SERVICE)
        .build();

    let app = application.clone();
    event_recv.attach(None, move |_event| {
        eprintln!(
            "Got event on tid={:?}, creating window",
            std::thread::current().id()
        );

        let mut windows = windows.borrow_mut();
        let win = create_window(&app);
        windows.push(win);

        Continue(true)
    });

    application.connect_activate(|_app| {
        eprintln!("Activated by gtk");
    });

    application.connect_shutdown(|_app| {
        eprintln!("Shutdown by gtk");
    });

    // Make sure app doesn't quit
    let _hold = application.hold();

    // Uncommenting this line makes the bug not reproduce!
    // sleep(Duration::from_millis(2000));

    application.run_with_args::<&str>(&[]);

    eprintln!("App done");
    std::process::exit(0);
}

fn create_window(app: &adw::Application) -> adw::Window {
    eprintln!("Creating window");
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let header_bar = adw::HeaderBar::new();
    content.append(&header_bar);

    let win = adw::Window::builder()
        .application(app)
        .title("It Works!")
        .default_width(300)
        .default_height(300)
        .content(&content)
        .build();

    win.show();
    win
}
