use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{terminal, ExecutableCommand, event};
use crossterm::cursor::{Hide, Show};
use rusty_audio::Audio;
use std::error::Error;
use std::io;
use std::os::unix::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::time::Duration;
use std::thread::spawn;
use invaders::{
    frame::{self, new_frame, Drawable, Frame},
    invaders::Invaders,
    level::Level,
    menu::Menu,
    player::Player,
    render,
    score::Score,
};

fn main() -> Result<(),Box<dyn Error>>{
    let mut audio = Audio::new();
    audio.add("explode","explode.wav");
    audio.add("lose","lose.wav");
    audio.add("move","move.wav");
    audio.add("pew","pew.wav");
    audio.add("startup","startup.wav");
    audio.add("win","win.wav");
    audio.play("startup");
    
 
    //Abrindo terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    //Render loop in a separete thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = spawn(move || {
        let mut last_frame = new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,

            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    //Game Loop
    'gameloop: loop {
        let curr_frame = new_frame();

        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {

                    }
                }
            }
        }

        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
    }


    //Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}