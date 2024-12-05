use std::{io, sync::mpsc::Receiver, time::Duration};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        self,
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::bar::Set,
    text::Span,
    widgets::{Block, BorderType, Sparkline},
    Frame, Terminal,
};

use crate::report::{Power, PowerStore};

pub struct App {
    rx: Receiver<Power>,
    data: PowerStore,
}

impl App {
    pub fn new(rx: Receiver<Power>) -> Self {
        Self {
            rx,
            data: PowerStore::new(200),
        }
    }

    pub fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if (key.code == KeyCode::Char('c')
                        && key.modifiers == event::KeyModifiers::CONTROL)
                        || key.code == KeyCode::Char('q')
                    {
                        break;
                    }
                }
            }
            if let Ok(pow) = self.rx.try_recv() {
                self.data.push_back(pow);
            }
            terminal.draw(|f| ui(f, &self))?;
        }
        Ok(())
    }
}

const DOTS: Set = Set {
    full: "⣿",
    seven_eighths: "⣷",
    three_quarters: "⣶",
    five_eighths: "⣦",
    half: "⣤",
    three_eighths: "⣄",
    one_quarter: "⣀",
    one_eighth: "⡀",
    empty: " ",
};

fn sparkline<'a>(title: &str, data: &'a [u64], border_color: Color) -> Sparkline<'a> {
    Sparkline::default()
        .block(
            Block::bordered()
                .title(
                    Span::from(format!("{} ({}mw)", title, data.last().unwrap_or(&0)))
                        .style(Style::new().fg(Color::White).bold().not_dim()),
                )
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color).dim()),
        )
        .bar_set(DOTS)
        .data(data)
}

#[derive(Default)]
struct ComponentRect {
    total: Rect,
    cpu: Rect,
    gpu: Rect,
    e_cpu: Option<Rect>,
    p_cpu: Option<Rect>,
    gpu_sram: Option<Rect>,
    dram: Option<Rect>,
    ane: Option<Rect>,
    ave: Option<Rect>,
    isp: Option<Rect>,
    pcie: Option<Rect>,
}

impl ComponentRect {
    pub fn new(total: Rect, cpu: Rect, gpu: Rect) -> Self {
        Self {
            total,
            cpu,
            gpu,
            ..Default::default()
        }
    }
}

pub fn ui(f: &mut Frame, app: &App) {
    let size = f.area();

    let component_rect = if size.height < 15 {
        let vertical: [Rect; 2] = Layout::vertical(Constraint::from_fills([2, 1])).areas(f.area());
        let [cpu, gpu] = Layout::horizontal(Constraint::from_fills([1, 1])).areas(vertical[1]);

        ComponentRect::new(vertical[0], cpu, gpu)
    } else {
        let vertical: [Rect; 4] =
            Layout::vertical(Constraint::from_fills([2, 1, 1, 1])).areas(f.area());

        let total = vertical[0];

        let [cpu, p_cpu, e_cpu] =
            Layout::horizontal(Constraint::from_fills([3, 1, 1])).areas(vertical[1]);

        let [gpu, gpu_sram] = Layout::horizontal(Constraint::from_fills([4, 1])).areas(vertical[2]);

        let [dram, ane, ave, isp, pcie] =
            Layout::horizontal(Constraint::from_fills([1, 1, 1, 1, 1])).areas(vertical[3]);

        ComponentRect {
            total,
            cpu,
            gpu,
            e_cpu: Some(e_cpu),
            p_cpu: Some(p_cpu),
            gpu_sram: Some(gpu_sram),
            dram: Some(dram),
            ane: Some(ane),
            ave: Some(ave),
            isp: Some(isp),
            pcie: Some(pcie),
        }
    };

    macro_rules! render {
        ($f:expr, $title:literal, $data:ident, $color:expr, $style:ident) => {
            $f.render_widget(
                sparkline(
                    $title,
                    &app.data.$data(component_rect.$data.width as usize),
                    $color,
                )
                .$style(),
                component_rect.$data,
            )
        };
        ($f:expr, $title:literal, if $data:ident, $color:expr, $style:ident) => {
            if let Some(data) = component_rect.$data {
                $f.render_widget(
                    sparkline($title, &app.data.$data(data.width as usize), $color).$style(),
                    data,
                )
            }
        };
    }

    render!(f, "Total", total, Color::LightGreen, green);
    render!(f, "CPU", cpu, Color::LightCyan, cyan);
    render!(f, "GPU", gpu, Color::LightBlue, blue);

    render!(f, "E CPU", if e_cpu, Color::LightCyan, cyan);
    render!(f, "P CPU", if p_cpu, Color::LightCyan, cyan);
    render!(f, "GPU SRAM", if gpu_sram, Color::LightBlue, blue);
    render!(f, "DRAM", if dram, Color::LightMagenta, magenta);
    render!(f, "ANE", if ane, Color::LightYellow, yellow);
    render!(f, "AVE", if ave, Color::LightYellow, yellow);
    render!(f, "ISP", if isp, Color::LightYellow, yellow);
    render!(f, "PCIe", if pcie, Color::LightYellow, yellow);
}

pub fn run_main(rx: Receiver<Power>) -> io::Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = App::new(rx).run(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
