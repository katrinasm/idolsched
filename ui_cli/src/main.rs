mod error;

use std::path::PathBuf;
use std::collections::BTreeMap;

use rand::SeedableRng;
use rand::rngs::SmallRng;

use idolsched::schedule::Schedule;
use idolsched::accessory::Acc;
use idolsched::card::Card;
use local_search::{SimpleIterSolver, anneal};
use card_consumer::Monicker;
use card_consumer::network::Cfg as ApiCfg;
use card_consumer::network::{get_cards, get_images};

// pretty sure there is a way to make Cargo give you this,
// not that it matters
const VERSION: &'static str = "experimental 2021-04-30";

// a lot of async code rn that really doesn't need to be async that badly.
// it will remain async for now since it is not really hurting anything
#[tokio::main]
async fn main() -> Result<(), error::Error> {
    use RunSettings::*;
    let run = get_configuration()?;
    match run {
        DoNothing => {},
        ShowVersion => {
            println!("SIFAS Schedule Optimizer {}", VERSION)
        },
        UpdateThumbnails { api_cfg, thumb_dir } => {
            get_images::thumbs(&thumb_dir, &api_cfg, None).await?;
        },
        UpdateCardData { api_cfg, data_path } => {
            update_card_data(&api_cfg, &data_path).await?;
        },
        Build(settings) => run_teambuild(settings).await?,
    };
    Ok(())
}

#[derive(Debug, Clone)]
enum RunSettings {
    DoNothing,
    ShowVersion,
    UpdateThumbnails {
        api_cfg: ApiCfg,
        thumb_dir: String,
    },
    UpdateCardData {
        api_cfg: ApiCfg,
        data_path: String,
    },
    Build(TbSettings),
}

#[derive(Debug, Clone)]
struct TbSettings {
    step_count: u32,
    acct_path: PathBuf,
    api_cfg: ApiCfg,
    map_override: Option<u32>,
}

async fn run_teambuild(settings: TbSettings) -> Result<(), error::Error> {
    let acct_json = std::fs::read_to_string(settings.acct_path)?;
    let acct = idolsched::init_acct(&acct_json)?;
    let (card_details, card_names) = get_cards::by_ordinal(&settings.api_cfg, acct.card_ordinals()).await?;
    let song_id = if let Some(v) = settings.map_override {
        v
    } else {
        1_0_015_30_1
    };
    let trimmed_details = card_consumer::trim_cards(&card_details);
    let song_json = std::fs::read_to_string(&format!("./mapdb/{}.json", song_id))?;
    let glob = idolsched::init_glob(&trimmed_details, &acct, song_id, &song_json)?;
    let mut rng = SmallRng::from_entropy();
    let s0 = Schedule::new_random(&mut rng, glob.album.len(), glob.inventory.len());
    let pm = anneal::Params { rng, t0: 10_000.0, alpha: 1.0 - (1.0/65_536.0) };
    let mut annealer = anneal::Annealer::org(s0, glob.clone(), pm);
    let (final_sched, energy) = run_showy(&mut annealer, settings.step_count);
    println!("Voltage est: {:.1}", -energy);
    display_sched(&glob.album, &glob.inventory, &final_sched, &card_names);
    Ok(())
}

fn run_showy<Sv: SimpleIterSolver<Schedule>>(solver: &mut Sv, steps: u32)
-> (Schedule, f64) {
    use std::io::Write;

    let one_percent = steps / 100;
    let mut best: Option<(Schedule, f64)> = None;

    let mut step = 0;
    let mut percent = 0;
    let mut countdown = 0;
    while let Some(new) = solver.advance() {
        if step == steps {
            break;
        }
        step += 1;

        if countdown == 0 {
            if let Some((_, e)) = best {
                print!("{:>3}% |    {:>12.1}    {:>12.1}\r", percent, -e, -new.1);
            } else {
                print!("{:>3}%\r", percent);
            }
            std::io::stdout().flush().unwrap();
            countdown = one_percent;
            percent += 1;
        } else {
            countdown -= 1;
        }

        if let Some(ref v) = best {
            if new.1 < v.1 {
                best = Some(new);
            }
        } else {
            best = Some(new);
        }
    }

    print!("                                                   \r");

    best.unwrap()
}

fn display_sched(album: &Vec<Card>, inv: &Vec<Acc>, sched: &Schedule, monickers: &BTreeMap<u32, Monicker>) {
    for (i, card_i) in sched.cards.iter().enumerate() {
        if i == 0 {
            println!("-- Green ---------------");
        } else if i % 3 == 0 {
            println!("------------------------");
        }
        let prefix = if sched.sp3[0] == i {
            '#'
        } else if sched.sp3[1] == i || sched.sp3[2] == i {
            '+'
        } else {
            '-'
        };

        let card = &album[*card_i];
        println!(" {} {:>3} {} ({})", prefix, card.ordinal, monickers.get(&card.ordinal).unwrap(), card_i);

        if i % 3 == 2 {
            let strat_accs = &sched.accs[i - 2 .. i + 1];
            for &acc_i in strat_accs.iter() {
                let acc = &inv[acc_i];
                if !acc.is_empty() {
                    print!(" | {}", acc.name());
                }
            }
            println!("");
        }
    }
}

// this function (as well as get_cfg) blocks on I/O bc we can't even start other I/O
// without the information it provides
fn get_configuration() -> Result<RunSettings, error::Error> {
    use RunSettings::*;
    let mut opts = getopts::Options::new();
    opts.optopt("n", "stepcount",
        "number of steps for the annealer.\n\
        needs enormous changes to see much difference;\n\
        if you are having poor results, trying adding 0s.\n\
        defaults to 10000 if unspecified.",
        "STEPS"
    );
    opts.optopt("", "api-cfg",
        "path to API configuration file.\n\
        defaults to 'api.json' if unspecified.",
        "FILE"
    );
    opts.optopt("a", "account",
        "path to a json file containing your account data (cards, accessories, bond).\n\
        defaults to 'account.json' if unspecified.",
        "FILE"
    );
    opts.optopt("m", "beatmap",
        "attribute override. if absent, the song's default attribute is used; if present,\n\
        attribute is replaced with 0123456 = XSPCANE,\n\
        where X is neutral, and SPCANE are the six main attributes. e.g.:\n\
        --attribute=4 chooses Active",
        "FILE"
    );
    opts.optflag("", "version", "print version information and exit immediately");
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("", "web-update-thumbnails",
        "if present, update thumbnail database in DIR instead of building a team.",
        "DIR"
    );
    opts.optopt("", "web-update-card-data",
        "if present, create a card file for use building the web UI instead of building a team.",
        "FILE"
    );

    let args: Vec<String> = std::env::args().skip(1).collect();
    let matches = opts.parse(&args)?;
    if matches.opt_present("h") {
        print!("{}", opts.usage("Usage: idolsched [options]"));
        return Ok(DoNothing);
    }

    if matches.opt_present("version") {
        return Ok(ShowVersion);
    }

    let cfg_path = matches.opt_str("api-cfg").unwrap_or_else(|| "api.json".to_string());
    let api_cfg = get_cfg(&cfg_path)?;

    if let Some(thumb_dir) = matches.opt_str("web-update-thumbnails") {
        return Ok(UpdateThumbnails {api_cfg, thumb_dir});
    }

    if let Some(data_path) = matches.opt_str("web-update-card-data") {
        return Ok(UpdateCardData {api_cfg, data_path});
    }

    let map_override = matches.opt_str("beatmap").map(|s| s.parse().unwrap());

    let acct_path = PathBuf::from(matches.opt_str("account").unwrap_or_else(|| "account.json".to_string()));

    let step_count: u32 = match matches.opt_get_default("n", 10000) {
        Ok(v) => v,
        Err(e) => return Err(error::Error::Etc(Box::new(e))),
    };

    Ok(Build(TbSettings { step_count, acct_path, api_cfg, map_override }))
}

fn get_cfg(path: &str) -> Result<ApiCfg, error::Error> {
    let f = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(f);
    let mut cfg: card_consumer::network::Cfg = serde_json::from_reader(reader)?;
    if !cfg.provider.ends_with('/') {
        cfg.provider.push('/');
    }
    Ok(cfg)
}

async fn update_card_data(api_cfg: &ApiCfg, data_path: &str) -> Result<(), error::Error> {
    let (cards, _names) = get_cards::til_latest(&api_cfg).await?;
    let trimmed = card_consumer::trim_cards(&cards);
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(data_path)?;
    serde_json::to_writer(&mut f, &trimmed)?;
    Ok(())
}

