import "./ui.js";
let wasm_promise = import("../pkg/index.js").catch(console.error);
import static_card_data from './cards.json';

const girl_names = {
    1:   'honoka',
    2:   'eli',
    3:   'kotori',
    4:   'umi',
    5:   'rin',
    6:   'maki',
    7:   'nozomi',
    8:   'hanayo',
    9:   'nico',
    101: 'chika',
    102: 'riko',
    103: 'kanan',
    104: 'dia',
    105: 'you',
    106: 'yohane',
    107: 'hanamaru',
    108: 'mari',
    109: 'ruby',
    201: 'ayumu',
    202: 'kasumi',
    203: 'shizuku',
    204: 'karin',
    205: 'ai',
    206: 'kanata',
    207: 'setsuna',
    208: 'emma',
    209: 'rina',
    210: 'shioriko',
};

const rarity_names = {
  10: 'r',
  20: 'sr',
  30: 'ur',
};

const attribute_names = {
  1: 'smile',
  2: 'pure',
  3: 'cool',
  4: 'active',
  5: 'natural',
  6: 'elegant',
  9: 'neutral',
};

function enumerate_girls() {
  let card_lemmas = {};
  let cards_by_lemma = {};
  let counters = {};
  for(const girl_id of Object.keys(girl_names)) {
    counters[girl_id] = {10: 0, 20: 0, 30: 0};
  }

  for(const card of Object.values(static_card_data)) {
    let girl_id = card.member;
    let rarity = card.rarity;
    counters[girl_id][rarity] += 1;
    let n = counters[girl_id][rarity];
    let lemma = `${girl_names[girl_id]}-${rarity_names[rarity]}${n}`;
    card_lemmas[card.ordinal] = lemma;
    cards_by_lemma[lemma] = card.ordinal;
  }
  return [counters, card_lemmas, cards_by_lemma];
};

let [card_counters, card_lemmas, cards_by_lemma] = enumerate_girls();

function account_as_json() {
  let display_account = { bond: account.bond, album: {}, accs: account.accs };
  for(let [ord, card] of account.album) {
    display_account.album[ord.toString()] = card;
  }
  return JSON.stringify(display_account);
}

async function fetch_song_text(song_id) {
  // this relies on song IDs never starting with 0
  const song_loc = `/mapdb/${song_id}.json`;
  const response = await fetch(song_loc);
  if(response.ok)
    return response.text();
  else
    throw "couldn't load song aaaa";
}

async function fetch_song_list() {
  const response = await fetch("./songlist.json");
  if(response.ok)
    return response.json();
  else
    throw "couldn't load song list aaaa";
}

async function run(step_input, song_sel) {
  insert_throbber();

  let song_id = null;
  for(const option of song_sel.selectedOptions)
    song_id = parseInt(option.value);
  const song_json = await fetch_song_text(song_id);
  const step_count = parseInt(step_input.value);

  // if(solve_worker)
    // run_worker(card_info, song_id, song_json, step_count);
  // else
    run_bourgeois(song_id, song_json, step_count);
}
/*
async function run_worker(card_info, song_id, song_json, step_count) {
  document.getElementById("run-button").disabled = true;
  solve_worker.postMessage({
    account: global_account,
    card_info: card_info,
    song_id: song_id,
    song_json: song_json,
    step_count: step_count,
    rnglo: random_u32(),
    rnghi: random_u32(),
  });
}
*/
async function run_bourgeois(song_id, song_json, step_count) {
  const account_json = account_as_json();
  let wasm = await(wasm_promise);
  const output_j = wasm.run_solver(step_count, JSON.stringify(static_card_data), account_json, song_id, song_json, random_u32(), random_u32());
  const output = JSON.parse(output_j);
  insert_schedule_display(output);
}

function random_u32() {
  return (Math.random() * 0x100000000)|0;
}

async function insert_throbber() {
  let inner_display = document.getElementById("display-area-team");
  if(inner_display.lastChild.className != "throbber")
    inner_display.replaceChild(make_throbber(), inner_display.lastChild);
}

async function insert_schedule_display(output) {
  const d = create_schedule_display(output);
  let inner_display = document.getElementById("display-area-team");
  inner_display.replaceChild(d, inner_display.lastChild);
}

function niceish_name(lemma) {
  let cap = lemma.charAt(0).toUpperCase() + lemma.slice(1);
  return cap.replace("-ur", "").replace("-sr", " SR").replace("-r", " R");
}

function create_schedule_display(output) {
  let outer = document.createElement("div");
  let span_vo = document.createElement("span");
  span_vo.innerText = "Estimated voltage: ";
  outer.appendChild(span_vo);
  let span_val = document.createElement("b");
  span_val.innerText = output.voltage.toFixed(1);
  outer.appendChild(span_val);
  outer.appendChild(document.createElement("br"));
  let table = document.createElement("table");
  table.className = "card_table"
  // let top_row = document.createElement("tr");
  // for(let i of [3,4,5, 0,1,2, 6,7,8]) {
    // let td = document.createElement("td");
    // td.className = ['strat_g', 'strat_r', 'strat_b'][(i/3)|0];
    // td.innerText = " ";
    // if(output.sp3[0] == i)
      // td.innerText = "SP Center";
    // else if(output.sp3[1] == i || output.sp3[2] == i)
      // td.innerText = "SP backup";
    // top_row.appendChild(td);
  // }
  let name_row = document.createElement("tr");
  let thumb_row = document.createElement("tr");
  for(let i of [3,4,5, 0,1,2, 6,7,8]) {
    let name_td = document.createElement("td");
    name_td.className = ['strat_g', 'strat_r', 'strat_b'][(i/3)|0];
    const card_ordinal = output.cards[i];
    let lemma = card_lemmas[card_ordinal];
    let ordinal_span = document.createElement("b");
    ordinal_span.innerText = `${card_ordinal} `;
    name_td.appendChild(ordinal_span);
    let name_span = document.createElement("span");
    name_span.innerText = niceish_name(lemma);
    name_td.appendChild(name_span);
    name_row.appendChild(name_td);

    let thumb_td = document.createElement("td");
    let thumb = document.createElement("img");
    let card = account.album.get(card_ordinal);
    let iz = card && card.idolized? '-i' : '';
    thumb.src = `/asset/thumbnail/${lemma}${iz}.png`;
    thumb_td.appendChild(thumb);
    thumb_row.appendChild(thumb_td);
  }
  let acc_row = document.createElement("tr");
  for(let i of [3,4,5, 0,1,2, 6,7,8]) {
    let td = document.createElement("td");
    const acc_i = output.accs[i];
    if(acc_i < account.accs.length) {
      const acc = account.accs[acc_i];
      const att = attribute_names[acc.attribute];
      const r = rarity_names[acc.rarity].toUpperCase();
      let top_span = document.createElement("b");
      top_span.innerText = `${r} ${att} ${acc.kind}`;
      let btm_span = document.createElement("span");
      btm_span.innerText = `LB${acc.lb} SL${acc.sl} Lv${acc.lv}`;
      td.appendChild(top_span);
      td.appendChild(document.createElement("br"));
      td.appendChild(btm_span);
    }
    acc_row.append(td);
  }
  // table.appendChild(top_row);
  table.appendChild(name_row);
  table.appendChild(thumb_row);
  table.appendChild(acc_row);
  outer.appendChild(table);
  return outer;
}

function empty_account() {
  let account = { bond: {}, album: new Map(), accs: [] };
  return account;
}
let account = empty_account();

function cardTable() {
  let table = document.createElement("table");
  let columns = 0;
  for(const row of Object.values(card_counters))
    for(const val of Object.values(row))
      columns = Math.max(columns, val);
  let tr = document.createElement("tr");
  for(let i = 0; i++ < columns;) {
    let th = document.createElement("th");
    th.innerText = i.toString();
    tr.appendChild(th);
  }
  table.appendChild(tr);

  for(const [girl_id, girl_name] of Object.entries(girl_names)) {
    for(const rarity of [10, 20, 30]) {
      let rarity_name = rarity_names[rarity];
      let tr = document.createElement("tr");
      let columns = card_counters[girl_id][rarity];
      for(let i = 0; i++ < columns;)
      {
        let td = document.createElement("td");
        let lemma = `${girl_name}-${rarity_name}${i}`;
        let inner = cardControl(cards_by_lemma[lemma]);
        td.appendChild(inner);
        tr.appendChild(td);
      }
      table.appendChild(tr);
    }
  }
  return table;
}

function cardControl(card_id) {
  let container = document.createElement("div");
  container.className = "card-container";
  let table = cardDisplayTable(card_id);
  container.appendChild(table);
  container.onclick = function(e) {cycleCardControl(container, card_id, e.shiftKey)};

  return container;
};

function cardDisplayTable(card_id) {
  let table = document.createElement("table");
  let lb = 0;
  let idolized = false;
  if(account.album.has(card_id)) {
    let card_status = account.album.get(card_id);
    lb = card_status.lb;
    idolized = card_status.idolized;
    table.style.opacity = 1.0;
  } else {
    table.style.opacity = 0.5;
  }

  let tr = document.createElement("tr");

  let card_thumb = document.createElement("img");
  let lemma = card_lemmas[card_id];
  let i = idolized? '-i' : '';
  card_thumb.src = `/asset/thumbnail/${lemma}${i}.png`;
  let td_thumb = document.createElement("td");
  td_thumb.appendChild(card_thumb);
  td_thumb.rowSpan = 6;
  tr.appendChild(td_thumb);

  for(let i = 5; i--;) {
    let td = document.createElement("td");
    td.style.background = `url("asset/icon/lb-${i < lb? 1 : 0}.png")`
    td.style.width = '24px';
    td.style.height = '24px';
    tr.appendChild(td);
    table.appendChild(tr);
    tr = document.createElement("tr");
  }

  let td = document.createElement("td");
  let idolize_ind = document.createElement("img");
  td.style.background = `url("asset/icon/idol-${idolized? 1 : 0}.png")`
  td.style.width = '24px';
  td.style.height = '24px';
  tr.appendChild(td);
  table.appendChild(tr);
  return table;
}

function cycleCardControl(container, card_id, fast) {
  if(!account.album.has(card_id)) {
    account.album.set(card_id, freshCard());
    if(fast) {
      account.album.get(card_id).idolized = true;
    }
  } else {
    let card_details = account.album.get(card_id);
    if(!card_details.idolized)
      card_details.idolized = true;
    else if(card_details.lb < 5)
      card_details.lb = fast? 5 : card_details.lb + 1;
    else
      account.album.delete(card_id);
  }

  let table = cardDisplayTable(card_id);
  container.lastChild.replaceWith(table);
}

function freshCard() {
  return {idolized: false, lb: 0};
}

async function init_solver() {
  let div = document.createElement("div");
  let diff_text = document.createElement("label");

  let song_sel = await make_song_selector("a");
  diff_text.innerText = "Difficulty: ";
  div.appendChild(diff_text);
  let diff_sel = make_diff_selector(song_sel);
  div.appendChild(diff_sel);
  div.appendChild(document.createElement("br"));
  let song_text = document.createElement("label");
  song_text.innerText = "Song: ";
  div.appendChild(song_text);
  div.appendChild(song_sel);
  div.appendChild(document.createElement("br"));
  let step_text = document.createElement("label");
  step_text.innerText = "Step count: ";
  step_text.title = "Higher step counts make better teams but take longer to run.";
  div.appendChild(step_text);
  let step_input = document.createElement("input");
  step_input.type = "number"
  step_input.value = 2000;
  div.appendChild(step_input);
  let btn = document.createElement("button");
  btn.onclick = function() { run(step_input, song_sel) };
  btn.innerText = "Run";
  btn.id = "run-button";
  btn.disable = false;
  div.appendChild(btn);
  div.appendChild(document.createElement("br"));
  let team_display = document.createElement("div");
  team_display.id = "display-area-team";
  team_display.appendChild(document.createElement("div"));
  div.appendChild(team_display);
  let display_area = document.getElementById("display-area");
  display_area.replaceChild(div, display_area.lastChild);
}

async function init_album() {
  let div = document.createElement("div");
  let title = document.createElement("h1");
  title.innerText = "Album Editor";
  div.appendChild(title);
  div.appendChild(document.createElement("br"));
  let card_table = cardTable();
  div.appendChild(card_table);
  let display_area = document.getElementById("display-area");
  display_area.replaceChild(div, display_area.lastChild);
}

async function init_accs() {
  let div = document.createElement("div");
  let title = document.createElement("h1");
  title.innerText = "Accessory Editor";
  div.appendChild(title);
  let acc_selector = make_acc_selector();
  div.appendChild(acc_selector);
  let display_area = document.getElementById("display-area");
  display_area.replaceChild(div, display_area.lastChild);
}

async function init_copypaste() {
  let div = document.createElement("div");
  let title = document.createElement("h1");
  title.innerText = "Copy/Paste Account";
  div.appendChild(title);
  div.appendChild(document.createElement("br"));
  let note = document.createElement("span");
  note.innerText = "Save this in a .txt somewhere until I learn how cookies work"
  div.appendChild(note);
  div.appendChild(document.createElement("br"));
  let textarea = document.createElement("textarea");
  textarea.cols = 80;
  textarea.rows = 10;
  textarea.innerText = account_as_json();
  let load_button = document.createElement("button");
  load_button.innerText = "Load";
  load_button.onclick = function() {load_pasted(textarea);};
  div.appendChild(load_button);
  div.appendChild(document.createElement("br"));
  div.appendChild(textarea);
  let display_area = document.getElementById("display-area");
  display_area.replaceChild(div, display_area.lastChild);
}

function load_pasted(textarea) {
  let s = textarea.value;
  let new_data = JSON.parse(s);
  for(let key of Object.keys(new_data)) {
    if(key != 'bond' && key != 'album' && key != 'accs') {
      alert("not a real account");
      return;
    }
  }
  if(typeof(new_data.bond) === 'undefined' || typeof(new_data.album) === 'undefined' || typeof(new_data.accs) === 'undefined') {
    alert("not a real account");
    return;
  }
  let new_album = new Map();
  for(let [key, card] of Object.entries(new_data.album)) {
    let ord = parseInt(key);
    new_album.set(ord, card);
  }
  account = { bond: new_data.bond, album: new_album, accs: new_data.accs };
}

async function make_song_selector(difficulty) {
  let sel = document.createElement("select");
  update_song_selector(sel, 'a').await;
  return sel;
}

function make_diff_selector(song_selector) {
  let sel = document.createElement("select");
  for(let [diff, name] of [["a", "Advanced"], ["i", "Intermediate"], ["b", "Beginner"]]) {
    let option = document.createElement("option");
    option.value = diff;
    option.innerText = name;
    sel.appendChild(option);
  }
  sel.onchange = function() {
    const diff = sel.selectedOptions.length == 1?
      sel.selectedOptions[0].value : null;
    if(diff)
      update_song_selector(song_selector, diff);
  };
  return sel;
}

async function update_song_selector(sel, difficulty) {
  for(let node = sel.lastChild; node; node = sel.lastChild)
    sel.removeChild(node);

  let song_list = await fetch_song_list();
  for(let [id, name] of Object.entries(song_list[difficulty])) {
    let option = document.createElement("option");
    option.value = parseInt(id);
    option.innerText = name;
    sel.appendChild(option);
  }
}

function make_throbber() {
    let img = document.createElement("img");
    img.src = "/asset/throb.gif";
    img.className = "throbber";
    return img;
}

const acc_kind_info = {
  "brooch":    {rarities: [10, 20, 30], attrs: [1,3,5]},
  "bracelet":  {rarities: [10, 20, 30], attrs: [1,4,6]},
  "necklace":  {rarities: [10, 20, 30], attrs: [4,5,6]},
  "choker":    {rarities: [30],         attrs: [1,6]},
  "belt":      {rarities: [30],         attrs: [3,5]},
  "bangle":    {rarities: [30],         attrs: [2,4]},
  "keychain":  {rarities: [10, 20, 30], attrs: [2,4,6]},
  "hairpin":   {rarities: [10, 20, 30], attrs: [2,3,5]},
  "earring":   {rarities: [10, 20, 30], attrs: [1,2,3]},
  "pouch":     {rarities: [10, 20, 30], attrs: [3,4,6]},
  "ribbon":    {rarities: [10, 20, 30], attrs: [1,2,5]},
  "wristband": {rarities: [10, 20, 30], attrs: [1,3,6]},
  "towel":     {rarities: [10, 20, 30], attrs: [2,4,5]},
};

const acc_max_lv = {10: 40, 20: 50, 30: 60};

function make_acc_selector(card_info) {
  let div = document.createElement("div");
  let kind_sel = document.createElement("select");
  for(const kind of Object.keys(acc_kind_info)) {
    let option = document.createElement("option");
    option.value = kind;
    option.innerText = kind;
    kind_sel.appendChild(option);
  }
  div.appendChild(kind_sel);

  let kind_cfg = document.createElement("div");
  kind_cfg.appendChild(document.createElement("div"));
  update_acc_kind_cfg(kind_sel, kind_cfg);

  kind_sel.onchange = function(e) { update_acc_kind_cfg(kind_sel, kind_cfg) };

  div.appendChild(kind_cfg);

  let insert_btn = document.createElement("button");
  insert_btn.innerText = "Insert";

  let acc_list = document.createElement("table");
  for(let i = 0; i < account.accs.length; i++)
    display_acc(acc_list, account.accs[i]);

  insert_btn.onclick = function() {insert_acc(acc_list, kind_sel)};
  div.appendChild(insert_btn);
  div.appendChild(document.createElement("br"));
  div.appendChild(acc_list);

  return div;
}

function update_acc_kind_cfg(kind_sel, kind_cfg) {
  let div = document.createElement("div");
  if(kind_sel.selectedOptions.length != 1)
    return;
  let kind = kind_sel.selectedOptions[0].value;
  if(!kind)
    return;

  let r_text = document.createElement("label");
  r_text.innerText = "Rarity: ";
  div.appendChild(r_text);
  let r = document.createElement("select");
  for(const rarity of acc_kind_info[kind].rarities) {
    let option = document.createElement("option");
    option.value = rarity;
    option.innerText = rarity_names[rarity];
    r.appendChild(option);
  }
  r.id = "acc-rarity-sel";
  div.appendChild(r);

  let attr_text = document.createElement("label");
  attr_text.innerText = "Attribute: ";
  div.appendChild(attr_text);
  let attr_sel = document.createElement("select");
  for(const attr of acc_kind_info[kind].attrs) {
    let option = document.createElement("option");
    option.value = attr;
    option.innerText = attribute_names[attr];
    attr_sel.appendChild(option);
  }
  attr_sel.id = "acc-attr-sel";
  div.appendChild(attr_sel);

  let lb_text = document.createElement("label");
  lb_text.innerText = "LB: ";
  div.appendChild(lb_text);
  let lb = document.createElement("input");
  lb.type = "number"
  lb.value = 0;
  lb.min = 0;
  lb.max = 5;
  lb.id = "acc-lb-input";
  div.appendChild(lb);

  let lv_text = document.createElement("label");
  lv_text.innerText = "Lv: ";
  div.appendChild(lv_text);
  let lv = document.createElement("input");
  lv.type = "number"
  lv.value = 60;
  lv.min = 1;
  lv.max = 60;
  lv.id = "acc-lv-input";
  div.appendChild(lv);

  let sl_text = document.createElement("label");
  sl_text.innerText = "Skill level: ";
  div.appendChild(sl_text);
  let sl = document.createElement("input");
  sl.type = "number"
  sl.value = 1;
  sl.min = 1;
  sl.max = 20;
  sl.id = "acc-sl-input";
  div.appendChild(sl);

  kind_cfg.replaceChild(div, kind_cfg.lastChild);
}

function insert_acc(acc_list, kind_sel) {
  let ordinal = 1;
  if(kind_sel.selectedOptions.length != 1)
    return;
  let kind = kind_sel.selectedOptions[0].value;

  let rarity_sel = document.getElementById("acc-rarity-sel");
  let attr_sel = document.getElementById("acc-attr-sel");
  let lb_input = document.getElementById("acc-lb-input");
  let lv_input = document.getElementById("acc-lv-input");
  let sl_input = document.getElementById("acc-sl-input");

  if (rarity_sel.selectedOptions.length != 1)
    return;
  let rarity = parseInt(rarity_sel.selectedOptions[0].value);

  if(attr_sel.selectedOptions.length != 1)
    return;
  let attr = parseInt(attr_sel.selectedOptions[0].value);

  const lb = Math.max(0, Math.min(5, parseInt(lb_input.value)));
  const lv = Math.max(0, Math.min(acc_max_lv[rarity], parseInt(lv_input.value)));
  const sl = Math.max(0, Math.min(20, parseInt(sl_input.value)));

  const acc = {'kind': kind, 'rarity': rarity, 'attribute': attr, 'lb': lb, 'lv': lv, 'sl': sl };
  account.accs.push(acc);

  display_acc(acc_list, acc);
}

function display_acc(acc_list, acc) {
  let tr = document.createElement("tr");
  let kill_td = document.createElement("td");
  let kill_button = document.createElement("button");
  kill_button.innerText = "Delete";
  kill_button.onclick = function() { remove_acc(tr) };
  kill_td.appendChild(kill_button);
  tr.appendChild(kill_td);
  let name_td = document.createElement("td");
  name_td.innerText = `${rarity_names[acc.rarity].toUpperCase()} ${attribute_names[acc.attribute]} ${acc.kind}`;
  tr.appendChild(name_td);
  let lb_td = document.createElement("td");
  lb_td.innerText = `LB ${acc.lb}`;
  tr.appendChild(lb_td);
  let lv_td = document.createElement("td");
  lv_td.innerText = `Lv ${acc.lv}`;
  tr.appendChild(lv_td);
  let sl_td = document.createElement("td");
  sl_td.innerText = `SL ${acc.sl}`;
  tr.appendChild(sl_td);
  acc_list.appendChild(tr);
}

function remove_acc(tr) {
  let table = tr.parentNode;
  let i = 0;
  let table_child = table.firstChild;
  while(!table_child.isSameNode(tr)) {
    i += 1;
    if(table_child)
      table_child = table_child.nextSibling;
    else
      return;
  }
  account.accs.splice(i, 1);
  table.removeChild(tr);
}

document.body.onload = function() {
  document.getElementById("button-solver").onclick = init_solver;
  document.getElementById("button-album").onclick = init_album;
  document.getElementById("button-accs").onclick = init_accs;
  document.getElementById("button-copypaste").onclick = init_copypaste;

  init_solver();
}

