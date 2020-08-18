mod cards_api;
mod state;
mod sim;
mod anneal;
mod error;

use std::collections::{HashMap, BTreeMap};
use rand::SeedableRng;
use rand::rngs::SmallRng;

use cards_api::Monicker;

pub use error::Error;

// pretty sure there is a way to make Cargo give you this,
// not that it matters
const VERSION: &'static str = "experimental 2020-07-20";

use std::path::PathBuf;

use sim::basic_data::Attribute;

// a lot of async code rn that really doesn't need to be async that badly.
// it will remain async for now since it is not really hurting anything
#[tokio::main]
async fn main() -> Result<(), error::Error> {
    if let Some(settings) = get_configuration()? {
        let acct = get_acct(&settings.acct_path)?;
        let card_ordinals = acct.card_ordinals();
        let (card_details, card_names) = cards_api::get_cards(&settings.api_cfg, card_ordinals).await?;
        let mut album = Vec::new();
        let mut struggle_map: HashMap<u32, sim::acct_info::CardInfo> = HashMap::new();
        for card_inf in acct.cards.iter() {
            struggle_map.insert(card_inf.ordinal, *card_inf);
        }

        for (ordinal, jcard) in card_details.iter() {
            if let Some(card_inf) = struggle_map.get(ordinal) {
                let card = sim::card::Card::instantiate_json(&jcard, card_inf.lb, card_inf.fed);
                album.push(card);
            }
        }

        let inventory = acct.accs.iter().map(|info| sim::accessory::Acc::from_info(info)).collect();
        let song = if let Some(default_attribute) = settings.att_override {
            println!("Attribute override: {:?}", default_attribute);
            sim::song::Song {default_attribute, .. sim::song::TEST_SONG}
        } else {
            sim::song::TEST_SONG.clone()
        };
        let glob = sim::PlayGlob { album, inventory, song };
        let mut rng = SmallRng::from_entropy();
        let s0 = sim::schedule::Schedule::new_random(&mut rng, glob.album.len(), glob.inventory.len());
        let (_steps, final_sched, energy) = anneal::anneal(&mut rng, &s0, &glob, settings.step_count, 5_000_000.0);
        println!("Voltage est: {:.1}", -energy);
        display_sched(&glob.album, &glob.inventory, &final_sched, &card_names);
    }
    Ok(())
}

fn display_sched(album: &Vec<sim::card::Card>, inv: &Vec<sim::accessory::Acc>, sched: &sim::schedule::Schedule, monickers: &BTreeMap<u32, Monicker>) {
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
        println!(" {} {:>3} {} (appeal: {})", prefix, card.ordinal, monickers.get(&card.ordinal).unwrap(), card.appeal);

        if i % 3 == 2 {
            let strat_accs = &sched.accs[i - 2 .. i + 1];
            for acc_han in strat_accs.iter() {
                if let Some(acc_i) = acc_han.to_index() {
                    print!(" | {}", inv[acc_i].name());
                }
            }
            println!("");
        }
    }
}

#[derive(Debug, Clone)]
struct RunSettings {
    step_count: u32,
    acct_path: PathBuf,
    api_cfg: cards_api::Cfg,
    att_override: Option<Attribute>,
}

// this function (as well as get_cfg) blocks on I/O bc we can't even start other I/O
// without the information it provides
fn get_configuration() -> Result<Option<RunSettings>, error::Error> {
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
    opts.optopt("c", "attribute",
        "attribute override. if absent, the song's default attribute is used; if present,\n\
        attribute is replaced with 0123456 = XSPCANE,\n\
        where X is neutral, and SPCANE are the six main attributes. e.g.:\n\
        --attribute=4 chooses Active",
        "FILE"
    );
    opts.optflag("", "version", "print version information and exit immediately");
    opts.optflag("h", "help", "print this help menu");

    let args: Vec<String> = std::env::args().skip(1).collect();
    let matches = opts.parse(&args)?;
    if matches.opt_present("h") {
        print!("{}", opts.usage("Usage: idolsched [options]"));
        return Ok(None);
    }
    if matches.opt_present("version") {
        println!("SIFAS Schedule Optimizer {}", VERSION);
        return Ok(None);
    }

    let att_override = if let Some(s) = matches.opt_str("attribute") {
        serde_json::from_str(&s).unwrap()
    } else {
        None
    };

    let cfg_path = matches.opt_str("api-cfg").unwrap_or_else(|| "api.json".to_string());
    let api_cfg = get_cfg(&cfg_path)?;

    let acct_path = PathBuf::from(matches.opt_str("account").unwrap_or_else(|| "account.json".to_string()));

    let step_count: u32 = match matches.opt_get_default("n", 10000) {
        Ok(v) => v,
        Err(e) => return Err(error::Error::Etc(Box::new(e))),
    };

    Ok(Some(RunSettings { step_count, acct_path, api_cfg, att_override }))
}

fn get_cfg(path: &str) -> Result<cards_api::Cfg, error::Error> {
    let f = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(f);
    let mut cfg: cards_api::Cfg = serde_json::from_reader(reader)?;
    if !cfg.provider.ends_with('/') {
        cfg.provider.push('/');
    }
    Ok(cfg)
}

fn get_acct<P: AsRef<std::path::Path>>(path: &P) -> Result<sim::acct_info::AcctInfo, error::Error> {
    let p = path.as_ref();
    println!("Using account in '{}'", p.to_string_lossy());
    let f = std::fs::File::open(p)?;
    let reader = std::io::BufReader::new(f);
    let mut acct: sim::acct_info::AcctInfo = serde_json::from_reader(reader)?;
    acct.force_valid();
    Ok(acct)
}

