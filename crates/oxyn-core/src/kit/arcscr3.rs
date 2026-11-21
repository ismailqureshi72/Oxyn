//! Screens S14 (Arc screens III): **Market terminal**, **Product list**, **Product detail**,
//! **Invoice studio** and **Sign-in split**. The first four sit on the Arc screen frame (see
//! [`super::arcscr`]); the sign-in is a split page of its own. Behaviour and layout only: every
//! company, ticker, product, client and line of copy is fictional and our own, and every date is
//! an absolute ISO date.
//!
//! The rules are plain functions over plain data, so the Dioxus screens, the GPUI elements and
//! [`Arc3Ui`] (the live state a GPUI view keeps) agree:
//! - Market: fictional symbols, a price series per range ([`series`]), a live tick
//!   ([`live_price`]), positions, open orders and an order ticket ([`Ticket`]) that checks cash and
//!   shares before it places a market or limit order ([`Market::place`]);
//! - Products: filter by collection, colour and price, sort, grid or list ([`Catalog`]); the
//!   detail page picks a colour and a size (sold-out sizes refuse), a quantity, and adds to the
//!   cart ([`Catalog::add`]);
//! - Invoice: line items to add, edit and remove with undo, tax and payment terms, live totals,
//!   Preview and Send with checks ([`Invoice`]);
//! - Sign-in: email and password checks, a wrong password, a lock after three tries, and SSO
//!   ([`SignIn`]).

pub use super::arcscr::{
    HUES, ME, ME_INI, PEOPLE, ScreenState, States, TODAY, initials, iso, thousands, weekday,
};

/// Trading desk states.
pub const MARKET_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("limit", "Limit order"),
    ("placed", "Order placed"),
    ("closed", "Market closed"),
];
/// Product list states.
pub const PLIST_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("filtered", "Filtered"),
    ("listview", "List view"),
    ("nomatch", "No matches"),
];
/// Product detail states.
pub const PDETAIL_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("nosize", "Pick a size"),
    ("soldout", "Sold out"),
    ("added", "Added to cart"),
];
/// Invoice desk states.
pub const INVOICE_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("editing", "Editing"),
    ("preview", "Preview"),
    ("sent", "Sent"),
];
/// Sign-in split states.
pub const SIGNIN_STATES: States = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("invalid", "Invalid"),
    ("wrong", "Wrong password"),
    ("signing", "Signing in"),
];

/// The five S14 slugs, in set order.
pub const S14_SLUGS: [&str; 5] = [
    "market-terminal",
    "product-list",
    "product-detail",
    "invoice-studio",
    "sign-in-split",
];

/// The state bar of an S14 screen by slug.
pub fn s14_states(slug: &str) -> States {
    match slug {
        "product-list" => PLIST_STATES,
        "product-detail" => PDETAIL_STATES,
        "invoice-studio" => INVOICE_STATES,
        "sign-in-split" => SIGNIN_STATES,
        _ => MARKET_STATES,
    }
}

/// The data state and the screen's own state for a state key (own is `""` for the three data
/// states).
pub fn arc3_split(k: &str) -> (ScreenState, &'static str) {
    const OWN: [&str; 15] = [
        "limit", "placed", "closed", "filtered", "listview", "nomatch", "nosize", "soldout",
        "added", "editing", "preview", "sent", "invalid", "wrong", "signing",
    ];
    match OWN.iter().find(|x| **x == k) {
        Some(x) => (ScreenState::Sample, x),
        None => (ScreenState::from_key(k), ""),
    }
}

/// Cents as dollars: `1234567` → `$12,345.67`; negative with a leading minus.
pub fn money(c: i64) -> String {
    let s = if c < 0 { "-" } else { "" };
    let a = c.unsigned_abs();
    format!("{s}${}.{:02}", thousands(a / 100), a % 100)
}

/// Cents as whole dollars: `1234567` → `$12,346`.
pub fn money0(c: i64) -> String {
    let s = if c < 0 { "-" } else { "" };
    let a = c.unsigned_abs();
    format!("{s}${}", thousands((a + 50) / 100))
}

/// A signed percent in hundredths: `127` → `+1.27%`.
pub fn pct(bp: i64) -> String {
    let s = if bp < 0 { "-" } else { "+" };
    let a = bp.unsigned_abs();
    format!("{s}{}.{:02}%", a / 100, a % 100)
}

fn lcg(seed: &mut u64) -> f64 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    ((*seed >> 33) as f64) / ((1u64 << 31) as f64)
}

// ───────────────────────────────────────────────────────────── market ──

/// The symbols: (ticker, name, last close in cents, hue). All fictional.
pub const SYMBOLS: [(&str, &str, i64, &str); 7] = [
    ("KSTR", "Kestrel Systems", 18_642, "blue"),
    ("NVLA", "Novala Labs", 9_215, "violet"),
    ("BRWK", "Brightwick Energy", 4_388, "amber"),
    ("TMBR", "Timberline Foods", 6_120, "green"),
    ("QLMR", "Quillmoor Robotics", 27_310, "teal"),
    ("HRBR", "Harbor Lane Bank", 5_274, "rose"),
    ("CDRX", "Cedarix Health", 11_906, "magenta"),
];

/// Chart ranges: (label, points).
pub const MK_RANGES: [(&str, usize); 5] =
    [("1D", 78), ("1W", 35), ("1M", 22), ("3M", 64), ("1Y", 52)];

/// The price series (cents) of symbol `s` over range `r`, oldest first; the last point is the
/// close.
pub fn series(s: usize, r: usize) -> Vec<i64> {
    let s = s % SYMBOLS.len();
    let r = r % MK_RANGES.len();
    let n = MK_RANGES[r].1;
    let close = SYMBOLS[s].2 as f64;
    let vol = [0.004, 0.01, 0.018, 0.03, 0.05][r];
    let mut seed = (s as u64 + 1) * 7919 + r as u64 * 104_729;
    let mut back = vec![close];
    let mut p = close;
    let drift = if s % 3 == 2 { -0.3 } else { 0.35 };
    for _ in 1..n {
        let z = lcg(&mut seed) - 0.5 + drift * 0.08;
        p /= 1.0 + z * vol;
        back.push(p);
    }
    back.reverse();
    back.into_iter().map(|x| x.round() as i64).collect()
}

/// The price of symbol `s` at live tick `t` (a small deterministic walk around the close).
pub fn live_price(s: usize, t: u64) -> i64 {
    let s = s % SYMBOLS.len();
    let close = SYMBOLS[s].2;
    if t == 0 {
        return close;
    }
    let mut seed = (s as u64 + 3) * 31 + t * 2_654_435_761;
    let w = lcg(&mut seed) - 0.5;
    let wave = ((t as f64) * 0.7 + s as f64).sin() * 0.0025;
    (close as f64 * (1.0 + wave + w * 0.002)).round() as i64
}

/// The previous close of symbol `s` (cents).
pub fn prev_close(s: usize) -> i64 {
    let v = series(s % SYMBOLS.len(), 1);
    v[v.len().saturating_sub(8)]
}

/// The change since the previous close: (cents, basis points).
pub fn change(s: usize, price: i64) -> (i64, i64) {
    let pc = prev_close(s).max(1);
    (price - pc, (price - pc) * 10_000 / pc)
}

/// Whether a series ends up (or flat) from where it began.
pub fn up(v: &[i64]) -> bool {
    v.last() >= v.first()
}

/// Ticket side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Buy.
    Buy,
    /// Sell.
    Sell,
}

/// An order on the ticket or in the book.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Order {
    /// Id.
    pub id: u32,
    /// Symbol index.
    pub sym: usize,
    /// Buy or sell.
    pub side: Side,
    /// Shares.
    pub qty: u32,
    /// Limit price in cents (`None` for market).
    pub limit: Option<i64>,
    /// When, as the screen shows it.
    pub at: String,
}

/// The order ticket being filled in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticket {
    /// Buy or sell.
    pub side: Side,
    /// Market (false) or limit (true).
    pub limit: bool,
    /// Shares.
    pub qty: u32,
    /// The limit price, cents.
    pub price: i64,
}

/// The account, positions, orders and the ticket.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Market {
    /// The open symbol.
    pub sym: usize,
    /// The chart range.
    pub range: usize,
    /// Cash in cents.
    pub cash: i64,
    /// Shares held per symbol.
    pub held: [u32; 7],
    /// Average cost per symbol, cents.
    pub cost: [i64; 7],
    /// Open limit orders.
    pub orders: Vec<Order>,
    /// Filled orders, newest first.
    pub fills: Vec<Order>,
    /// The ticket.
    pub ticket: Ticket,
    /// The live tick (0 is the close).
    pub tick: u64,
    /// The market is open.
    pub open: bool,
    next: u32,
}

impl Market {
    /// The sample account (`sample`) or an empty one.
    pub fn new(sample: bool) -> Self {
        let mut held = [0; 7];
        let mut cost = [0; 7];
        let mut orders = Vec::new();
        if sample {
            held = [40, 120, 0, 75, 12, 0, 30];
            cost = [15_910, 8_402, 0, 6_355, 24_870, 0, 12_440];
            orders.push(Order {
                id: 1,
                sym: 1,
                side: Side::Buy,
                qty: 25,
                limit: Some(8_950),
                at: "2026-11-12 09:41".into(),
            });
            orders.push(Order {
                id: 2,
                sym: 4,
                side: Side::Sell,
                qty: 4,
                limit: Some(28_400),
                at: "2026-11-11 15:12".into(),
            });
        }
        Self {
            sym: 0,
            range: 0,
            cash: if sample { 2_418_650 } else { 0 },
            held,
            cost,
            orders,
            fills: Vec::new(),
            ticket: Ticket {
                side: Side::Buy,
                limit: false,
                qty: 10,
                price: SYMBOLS[0].2,
            },
            tick: 0,
            open: true,
            next: 3,
        }
    }

    /// The live price of the open symbol.
    pub fn price(&self) -> i64 {
        live_price(self.sym, self.tick)
    }

    /// The live price of symbol `s`.
    pub fn price_of(&self, s: usize) -> i64 {
        live_price(s, self.tick)
    }

    /// The account value: cash plus positions at live prices.
    pub fn value(&self) -> i64 {
        self.cash
            + (0..SYMBOLS.len())
                .map(|s| self.held[s] as i64 * self.price_of(s))
                .sum::<i64>()
    }

    /// The day's change in basis points of the account value.
    pub fn day_bp(&self) -> i64 {
        let before: i64 = self.cash
            + (0..SYMBOLS.len())
                .map(|s| self.held[s] as i64 * prev_close(s))
                .sum::<i64>();
        if before <= 0 {
            0
        } else {
            (self.value() - before) * 10_000 / before
        }
    }

    /// Opens symbol `s`; the ticket's limit price follows it.
    pub fn open_sym(&mut self, s: usize) -> String {
        self.sym = s % SYMBOLS.len();
        self.ticket.price = self.price();
        format!("{} · {}", SYMBOLS[self.sym].0, money(self.price()))
    }

    /// Moves the live tick on by one.
    pub fn step(&mut self) {
        if self.open {
            self.tick += 1;
        }
    }

    /// The ticket's estimated cost or proceeds, cents.
    pub fn estimate(&self) -> i64 {
        let p = if self.ticket.limit {
            self.ticket.price
        } else {
            self.price()
        };
        p * self.ticket.qty as i64
    }

    /// Why the ticket cannot be placed, if it cannot.
    pub fn check(&self) -> Option<String> {
        let t = &self.ticket;
        if t.qty == 0 {
            return Some("Enter a number of shares".into());
        }
        if t.limit && t.price <= 0 {
            return Some("Enter a limit price".into());
        }
        match t.side {
            Side::Buy if self.estimate() > self.cash => {
                Some(format!("Not enough cash: {} available", money(self.cash)))
            }
            Side::Sell if t.qty > self.held[self.sym] => Some(format!(
                "You hold {} {}",
                self.held[self.sym], SYMBOLS[self.sym].0
            )),
            _ => None,
        }
    }

    /// Places the ticket: a market order fills at the live price, a limit order joins the book.
    pub fn place(&mut self) -> Result<String, String> {
        if let Some(e) = self.check() {
            return Err(e);
        }
        let t = self.ticket.clone();
        let o = Order {
            id: self.next,
            sym: self.sym,
            side: t.side,
            qty: t.qty,
            limit: t.limit.then_some(t.price),
            at: format!("{} 14:05", iso(TODAY)),
        };
        self.next += 1;
        let verb = if t.side == Side::Buy { "Buy" } else { "Sell" };
        let tk = SYMBOLS[self.sym].0;
        if t.limit {
            self.orders.insert(0, o);
            return Ok(format!(
                "Limit order placed: {verb} {} {tk} at {}",
                t.qty,
                money(t.price)
            ));
        }
        if !self.open {
            self.orders.insert(0, o);
            return Ok(format!("Queued for the open: {verb} {} {tk}", t.qty));
        }
        let p = self.price();
        let s = self.sym;
        match t.side {
            Side::Buy => {
                let h = self.held[s] as i64;
                self.cost[s] = (self.cost[s] * h + p * t.qty as i64) / (h + t.qty as i64);
                self.held[s] += t.qty;
                self.cash -= p * t.qty as i64;
            }
            Side::Sell => {
                self.held[s] -= t.qty;
                self.cash += p * t.qty as i64;
                if self.held[s] == 0 {
                    self.cost[s] = 0;
                }
            }
        }
        self.fills.insert(0, o);
        Ok(format!("Filled: {verb} {} {tk} at {}", t.qty, money(p)))
    }

    /// Cancels open order `id`.
    pub fn cancel(&mut self, id: u32) -> Option<String> {
        let i = self.orders.iter().position(|o| o.id == id)?;
        let o = self.orders.remove(i);
        Some(format!("Cancelled {} {}", o.qty, SYMBOLS[o.sym].0))
    }

    /// Positions: (symbol, shares, average cost, live price).
    pub fn positions(&self) -> Vec<(usize, u32, i64, i64)> {
        (0..SYMBOLS.len())
            .filter(|&s| self.held[s] > 0)
            .map(|s| (s, self.held[s], self.cost[s], self.price_of(s)))
            .collect()
    }
}

// ───────────────────────────────────────────────────────────── products ──

/// Collections: (name, hue).
pub const COLLS: [(&str, &str); 4] = [
    ("Outerwear", "blue"),
    ("Knitwear", "amber"),
    ("Footwear", "teal"),
    ("Bags", "violet"),
];

/// Colours: (name, swatch hex).
pub const COLOURS: [(&str, &str); 6] = [
    ("Charcoal", "#3A3A3F"),
    ("Oat", "#D9CBB0"),
    ("Moss", "#5E6E45"),
    ("Rust", "#A4522F"),
    ("Navy", "#26324F"),
    ("Chalk", "#EFEDE7"),
];

/// Sizes.
pub const SIZES: [&str; 5] = ["XS", "S", "M", "L", "XL"];

/// Sorts: (key, label).
pub const SORTS: [(&str, &str); 4] = [
    ("featured", "Featured"),
    ("low", "Price, low to high"),
    ("high", "Price, high to low"),
    ("new", "Newest"),
];

/// A product.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Product {
    /// Id.
    pub id: u32,
    /// Name.
    pub name: &'static str,
    /// Collection index.
    pub coll: usize,
    /// Price in cents.
    pub price: i64,
    /// Was price (on sale) in cents.
    pub was: Option<i64>,
    /// Colour indices.
    pub colours: &'static [usize],
    /// Stock per size (XS … XL).
    pub stock: [u32; 5],
    /// Rating in tenths (46 = 4.6).
    pub rating: u32,
    /// Reviews.
    pub reviews: u32,
    /// Added on demo day.
    pub added: i32,
    /// One line about it.
    pub blurb: &'static str,
}

impl Product {
    /// Total stock.
    pub fn total(&self) -> u32 {
        self.stock.iter().sum()
    }
}

/// The catalogue (`sample`) or none.
pub fn products(sample: bool) -> Vec<Product> {
    if !sample {
        return Vec::new();
    }
    let p = |id,
             name,
             coll,
             price,
             was,
             colours: &'static [usize],
             stock,
             rating,
             reviews,
             added,
             blurb| Product {
        id,
        name,
        coll,
        price,
        was,
        colours,
        stock,
        rating,
        reviews,
        added,
        blurb,
    };
    vec![
        p(
            1,
            "Fellside Wool Coat",
            0,
            28_900,
            None,
            &[0, 1, 4],
            [2, 6, 0, 4, 1],
            47,
            212,
            30,
            "A long coat in brushed wool with a quilted lining.",
        ),
        p(
            2,
            "Tarn Rain Shell",
            0,
            16_500,
            Some(19_500),
            &[2, 4, 0],
            [5, 8, 9, 3, 2],
            45,
            98,
            36,
            "A light shell that packs into its own pocket.",
        ),
        p(
            3,
            "Ridgeway Puffer",
            0,
            22_400,
            None,
            &[0, 3],
            [0, 0, 3, 2, 0],
            44,
            61,
            40,
            "Recycled fill, a two-way zip and a stowable hood.",
        ),
        p(
            4,
            "Hollin Crew Knit",
            1,
            9_800,
            None,
            &[1, 2, 5],
            [6, 10, 12, 7, 3],
            48,
            340,
            12,
            "A crew neck in soft merino, cut for layering.",
        ),
        p(
            5,
            "Brae Cable Cardigan",
            1,
            12_400,
            Some(14_800),
            &[1, 3],
            [1, 3, 2, 0, 0],
            46,
            77,
            25,
            "A chunky cable cardigan with horn buttons.",
        ),
        p(
            6,
            "Linnet Rib Beanie",
            1,
            3_200,
            None,
            &[0, 3, 2, 4],
            [0, 0, 24, 0, 0],
            43,
            150,
            41,
            "A rib-knit beanie with a turned-up cuff.",
        ),
        p(
            7,
            "Scree Trail Runner",
            2,
            13_800,
            None,
            &[0, 2],
            [3, 5, 6, 4, 2],
            45,
            188,
            34,
            "A grippy trail shoe with a rock plate.",
        ),
        p(
            8,
            "Crag Leather Boot",
            2,
            24_500,
            None,
            &[3, 0],
            [0, 1, 2, 1, 0],
            49,
            64,
            20,
            "A welted boot in waxed leather, resoleable.",
        ),
        p(
            9,
            "Beck Canvas Tote",
            3,
            5_800,
            None,
            &[1, 4, 5],
            [0, 0, 30, 0, 0],
            44,
            120,
            38,
            "A heavy canvas tote with an inner zip pocket.",
        ),
        p(
            10,
            "Holm Day Pack",
            3,
            11_200,
            Some(12_900),
            &[0, 2, 4],
            [0, 0, 7, 0, 0],
            46,
            203,
            28,
            "A 22 litre pack with a padded laptop sleeve.",
        ),
    ]
}

/// The catalogue and cart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    /// Products.
    pub items: Vec<Product>,
    /// Collection filter.
    pub coll: Option<usize>,
    /// Colour filter.
    pub colour: Option<usize>,
    /// Price cap in cents.
    pub max: Option<i64>,
    /// Search text.
    pub query: String,
    /// Sort index.
    pub sort: usize,
    /// Grid (false) or list (true).
    pub list: bool,
    /// The open product (detail).
    pub open: Option<u32>,
    /// Picked colour on the detail.
    pub pick_colour: usize,
    /// Picked size on the detail.
    pub pick_size: Option<usize>,
    /// Quantity on the detail.
    pub qty: u32,
    /// Cart lines: (product, colour, size, quantity).
    pub cart: Vec<(u32, usize, usize, u32)>,
    /// The detail's last refusal.
    pub error: Option<String>,
}

impl Catalog {
    /// The sample catalogue (`sample`) or an empty shop.
    pub fn new(sample: bool) -> Self {
        Self {
            items: products(sample),
            coll: None,
            colour: None,
            max: None,
            query: String::new(),
            sort: 0,
            list: false,
            open: None,
            pick_colour: 0,
            pick_size: None,
            qty: 1,
            cart: Vec::new(),
            error: None,
        }
    }

    /// Whether any filter or search is on.
    pub fn filtered(&self) -> bool {
        self.coll.is_some()
            || self.colour.is_some()
            || self.max.is_some()
            || !self.query.trim().is_empty()
    }

    /// The products shown, filtered and sorted.
    pub fn shown(&self) -> Vec<Product> {
        let q = self.query.trim().to_lowercase();
        let mut v: Vec<Product> = self
            .items
            .iter()
            .filter(|p| self.coll.is_none_or(|c| p.coll == c))
            .filter(|p| self.colour.is_none_or(|c| p.colours.contains(&c)))
            .filter(|p| self.max.is_none_or(|m| p.price <= m))
            .filter(|p| q.is_empty() || p.name.to_lowercase().contains(&q))
            .cloned()
            .collect();
        match self.sort {
            1 => v.sort_by_key(|p| (p.price, p.id)),
            2 => v.sort_by_key(|p| (-p.price, p.id)),
            3 => v.sort_by_key(|p| (-p.added, p.id)),
            _ => {}
        }
        v
    }

    /// Clears every filter and the search.
    pub fn clear(&mut self) -> String {
        self.coll = None;
        self.colour = None;
        self.max = None;
        self.query.clear();
        "Filters cleared".into()
    }

    /// Opens product `id` on its first colour, no size picked.
    pub fn open_id(&mut self, id: u32) -> Option<String> {
        let p = self.items.iter().find(|p| p.id == id)?;
        self.open = Some(id);
        self.pick_colour = p.colours[0];
        self.pick_size = None;
        self.qty = 1;
        self.error = None;
        Some(p.name.to_string())
    }

    /// The open product.
    pub fn current(&self) -> Option<&Product> {
        self.items.iter().find(|p| Some(p.id) == self.open)
    }

    /// Picks size `s`; refuses a sold-out size.
    pub fn size(&mut self, s: usize) -> String {
        let Some(p) = self.current() else {
            return String::new();
        };
        let s = s.min(4);
        if p.stock[s] == 0 {
            let m = format!("{} is sold out", SIZES[s]);
            self.error = Some(m.clone());
            return m;
        }
        let left = p.stock[s];
        self.pick_size = Some(s);
        self.qty = self.qty.min(left).max(1);
        self.error = None;
        if left <= 3 {
            format!("Size {}: only {left} left", SIZES[s])
        } else {
            format!("Size {}", SIZES[s])
        }
    }

    /// Steps the quantity by `d` within the stock of the picked size (1 … 10).
    pub fn step_qty(&mut self, d: i32) -> u32 {
        let cap = match (self.current(), self.pick_size) {
            (Some(p), Some(s)) => p.stock[s].min(10),
            _ => 10,
        };
        self.qty = (self.qty as i32 + d).clamp(1, cap.max(1) as i32) as u32;
        self.qty
    }

    /// Adds the picked colour, size and quantity to the cart.
    pub fn add(&mut self) -> Result<String, String> {
        let Some(p) = self.current().cloned() else {
            return Err("Open a product first".into());
        };
        let Some(s) = self.pick_size else {
            let e = "Pick a size".to_string();
            self.error = Some(e.clone());
            return Err(e);
        };
        let have: u32 = self
            .cart
            .iter()
            .filter(|l| l.0 == p.id && l.2 == s)
            .map(|l| l.3)
            .sum();
        if have + self.qty > p.stock[s] {
            let e = format!("Only {} in {}", p.stock[s], SIZES[s]);
            self.error = Some(e.clone());
            return Err(e);
        }
        match self
            .cart
            .iter_mut()
            .find(|l| l.0 == p.id && l.1 == self.pick_colour && l.2 == s)
        {
            Some(l) => l.3 += self.qty,
            None => self.cart.push((p.id, self.pick_colour, s, self.qty)),
        }
        self.error = None;
        Ok(format!(
            "Added {} × {}, {} {}",
            self.qty, p.name, COLOURS[self.pick_colour].0, SIZES[s]
        ))
    }

    /// Items in the cart.
    pub fn cart_count(&self) -> u32 {
        self.cart.iter().map(|l| l.3).sum()
    }

    /// The cart subtotal, cents.
    pub fn cart_total(&self) -> i64 {
        self.cart
            .iter()
            .filter_map(|l| {
                self.items
                    .iter()
                    .find(|p| p.id == l.0)
                    .map(|p| p.price * l.3 as i64)
            })
            .sum()
    }
}

/// A stock word and hue: `Sold out` rose, `3 left` amber, `In stock` green.
pub fn stock_word(n: u32) -> (String, &'static str) {
    match n {
        0 => ("Sold out".into(), "rose"),
        1..=5 => (format!("{n} left"), "amber"),
        _ => ("In stock".into(), "green"),
    }
}

// ───────────────────────────────────────────────────────────── invoice ──

/// Clients: (name, email, hue).
pub const CLIENTS: [(&str, &str, &str); 4] = [
    ("Larkspur Studio", "accounts@larkspur.example", "violet"),
    ("Ottery Mills", "billing@ottery.example", "amber"),
    ("Pellow & Finch", "pay@pellowfinch.example", "teal"),
    ("Saltmarsh Co-op", "", "green"),
];

/// Payment terms: (label, days).
pub const TERMS: [(&str, i32); 4] = [
    ("On receipt", 0),
    ("Net 15", 15),
    ("Net 30", 30),
    ("Net 45", 45),
];

/// Invoice statuses: (label, hue).
pub const IV_STATUS: [(&str, &str); 4] = [
    ("Draft", "mute"),
    ("Open", "blue"),
    ("Overdue", "rose"),
    ("Paid", "green"),
];

/// A line item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// Id.
    pub id: u32,
    /// What it is.
    pub desc: String,
    /// Quantity (hours or units).
    pub qty: u32,
    /// Unit price, cents.
    pub unit: i64,
}

impl Line {
    /// The line's amount, cents.
    pub fn amount(&self) -> i64 {
        self.qty as i64 * self.unit
    }
}

/// An invoice in the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inv {
    /// Number (`NS-1042`).
    pub no: u32,
    /// Client index.
    pub client: usize,
    /// Status index into [`IV_STATUS`].
    pub status: usize,
    /// Issued on demo day.
    pub issued: i32,
    /// Terms index.
    pub terms: usize,
    /// Line items.
    pub lines: Vec<Line>,
    /// Tax in basis points (825 = 8.25%).
    pub tax: u32,
}

impl Inv {
    /// Subtotal, cents.
    pub fn subtotal(&self) -> i64 {
        self.lines.iter().map(Line::amount).sum()
    }
    /// Tax, cents (rounded half up).
    pub fn tax_amount(&self) -> i64 {
        (self.subtotal() * self.tax as i64 + 5_000) / 10_000
    }
    /// Total, cents.
    pub fn total(&self) -> i64 {
        self.subtotal() + self.tax_amount()
    }
    /// The due date (demo day).
    pub fn due(&self) -> i32 {
        self.issued + TERMS[self.terms.min(3)].1
    }
}

/// The invoice list (`sample`) or none.
pub fn invoices(sample: bool) -> Vec<Inv> {
    if !sample {
        return Vec::new();
    }
    let l = |id, d: &str, qty, unit| Line {
        id,
        desc: d.to_string(),
        qty,
        unit,
    };
    vec![
        Inv {
            no: 1046,
            client: 0,
            status: 0,
            issued: TODAY,
            terms: 2,
            lines: vec![
                l(1, "Brand workshop, two days", 2, 180_000),
                l(2, "Logo refinement", 14, 12_500),
                l(3, "Type licence (desktop, 5 seats)", 1, 34_900),
            ],
            tax: 825,
        },
        Inv {
            no: 1045,
            client: 1,
            status: 1,
            issued: TODAY - 9,
            terms: 2,
            lines: vec![l(1, "Packaging system, phase 1", 1, 640_000)],
            tax: 0,
        },
        Inv {
            no: 1043,
            client: 2,
            status: 2,
            issued: TODAY - 40,
            terms: 1,
            lines: vec![
                l(1, "Website art direction", 20, 15_000),
                l(2, "Illustration set", 6, 42_000),
            ],
            tax: 825,
        },
        Inv {
            no: 1041,
            client: 3,
            status: 3,
            issued: TODAY - 30,
            terms: 0,
            lines: vec![l(1, "Annual report layout", 1, 485_000)],
            tax: 0,
        },
    ]
}

/// The invoice desk: the list and the open invoice being edited.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invoice {
    /// Invoices.
    pub list: Vec<Inv>,
    /// The open invoice number.
    pub open: Option<u32>,
    /// The line being edited.
    pub edit: Option<u32>,
    /// The last removed line (for undo): (position, line).
    pub removed: Option<(usize, Line)>,
    /// Preview mode.
    pub preview: bool,
    /// Automatic reminders on.
    pub remind: bool,
}

impl Invoice {
    /// The sample studio (`sample`) or an empty one.
    pub fn new(sample: bool) -> Self {
        let list = invoices(sample);
        let open = list.first().map(|i| i.no);
        Self {
            list,
            open,
            edit: None,
            removed: None,
            preview: false,
            remind: true,
        }
    }

    /// The open invoice.
    pub fn current(&self) -> Option<&Inv> {
        self.list.iter().find(|i| Some(i.no) == self.open)
    }

    fn cur_mut(&mut self) -> Option<&mut Inv> {
        let n = self.open?;
        self.list.iter_mut().find(|i| i.no == n)
    }

    /// Whether the open invoice can still be edited (drafts only).
    pub fn editable(&self) -> bool {
        self.current().is_some_and(|i| i.status == 0)
    }

    /// Opens invoice `no`.
    pub fn open_no(&mut self, no: u32) -> Option<String> {
        let i = self.list.iter().find(|i| i.no == no)?;
        let s = format!("NS-{} · {}", no, CLIENTS[i.client].0);
        self.open = Some(no);
        self.edit = None;
        self.removed = None;
        self.preview = false;
        Some(s)
    }

    /// Starts a new draft for client 0 and opens it.
    pub fn new_draft(&mut self) -> String {
        let no = self.list.iter().map(|i| i.no).max().unwrap_or(1040) + 1;
        self.list.insert(
            0,
            Inv {
                no,
                client: 0,
                status: 0,
                issued: TODAY,
                terms: 2,
                lines: Vec::new(),
                tax: 0,
            },
        );
        self.open = Some(no);
        self.preview = false;
        self.edit = None;
        format!("Draft NS-{no}")
    }

    /// Adds an empty line and edits it.
    pub fn add_line(&mut self) -> Option<String> {
        if !self.editable() {
            return None;
        }
        let inv = self.cur_mut()?;
        let id = inv.lines.iter().map(|l| l.id).max().unwrap_or(0) + 1;
        inv.lines.push(Line {
            id,
            desc: String::new(),
            qty: 1,
            unit: 0,
        });
        self.edit = Some(id);
        Some("Line added".into())
    }

    /// Sets line `id`'s fields (any `None` leaves it).
    pub fn set_line(
        &mut self,
        id: u32,
        desc: Option<&str>,
        qty: Option<u32>,
        unit: Option<i64>,
    ) -> bool {
        if !self.editable() {
            return false;
        }
        let Some(l) = self
            .cur_mut()
            .and_then(|i| i.lines.iter_mut().find(|l| l.id == id))
        else {
            return false;
        };
        if let Some(d) = desc {
            l.desc = d.to_string();
        }
        if let Some(q) = qty {
            l.qty = q.min(9_999);
        }
        if let Some(u) = unit {
            l.unit = u.clamp(0, 99_999_999);
        }
        true
    }

    /// Removes line `id` (undo puts it back).
    pub fn remove_line(&mut self, id: u32) -> Option<String> {
        if !self.editable() {
            return None;
        }
        let inv = self.cur_mut()?;
        let at = inv.lines.iter().position(|l| l.id == id)?;
        let l = inv.lines.remove(at);
        let s = format!(
            "Removed {}",
            if l.desc.is_empty() {
                "a line"
            } else {
                l.desc.as_str()
            }
        );
        self.removed = Some((at, l));
        if self.edit == Some(id) {
            self.edit = None;
        }
        Some(s)
    }

    /// Puts the last removed line back.
    pub fn undo(&mut self) -> Option<String> {
        let (at, l) = self.removed.take()?;
        let inv = self.cur_mut()?;
        let at = at.min(inv.lines.len());
        inv.lines.insert(at, l);
        Some("Line restored".into())
    }

    /// Sets the payment terms.
    pub fn terms(&mut self, t: usize) -> Option<String> {
        if !self.editable() {
            return None;
        }
        let inv = self.cur_mut()?;
        inv.terms = t.min(3);
        Some(format!("{} · due {}", TERMS[inv.terms].0, iso(inv.due())))
    }

    /// Turns tax on (8.25%) or off.
    pub fn toggle_tax(&mut self) -> Option<String> {
        if !self.editable() {
            return None;
        }
        let inv = self.cur_mut()?;
        inv.tax = if inv.tax == 0 { 825 } else { 0 };
        Some(if inv.tax == 0 {
            "Sales tax off".into()
        } else {
            "Sales tax 8.25%".into()
        })
    }

    /// Sets the client.
    pub fn client(&mut self, c: usize) -> Option<String> {
        if !self.editable() {
            return None;
        }
        let inv = self.cur_mut()?;
        inv.client = c % CLIENTS.len();
        Some(CLIENTS[inv.client].0.to_string())
    }

    /// Why the open invoice cannot be sent, if it cannot.
    pub fn check(&self) -> Option<&'static str> {
        let i = self.current()?;
        if i.status != 0 {
            return Some("Already sent");
        }
        if CLIENTS[i.client].1.is_empty() {
            return Some("Add the client's billing email");
        }
        if i.lines.is_empty() {
            return Some("Add a line item");
        }
        if i.lines.iter().any(|l| l.desc.trim().is_empty()) {
            return Some("Describe every line");
        }
        if i.total() <= 0 {
            return Some("The total is zero");
        }
        None
    }

    /// Sends the open invoice (it becomes Open).
    pub fn send(&mut self) -> Result<String, String> {
        if let Some(e) = self.check() {
            return Err(e.to_string());
        }
        let inv = self.cur_mut().ok_or("Open an invoice")?;
        inv.status = 1;
        let s = format!(
            "NS-{} sent to {} · {} due {}",
            inv.no,
            CLIENTS[inv.client].1,
            money(inv.total()),
            iso(inv.due())
        );
        self.preview = false;
        self.edit = None;
        Ok(s)
    }

    /// Count by status.
    pub fn count(&self, s: usize) -> usize {
        self.list.iter().filter(|i| i.status == s).count()
    }

    /// Outstanding (open + overdue), cents.
    pub fn outstanding(&self) -> i64 {
        self.list
            .iter()
            .filter(|i| i.status == 1 || i.status == 2)
            .map(Inv::total)
            .sum()
    }
}

/// `NS-1046`.
pub fn inv_no(n: u32) -> String {
    format!("NS-{n}")
}

// ───────────────────────────────────────────────────────────── sign-in ──

/// The demo account that signs in.
pub const DEMO_EMAIL: &str = "rowan@harrowgate.example";
/// Its password.
pub const DEMO_PASS: &str = "fern-lantern-42";

/// SSO providers: (brand id, label).
pub const SSO: [(&str, &str); 2] = [("google", "Google"), ("github", "GitHub")];

/// Why an email is not valid, if it is not.
pub fn email_error(e: &str) -> Option<&'static str> {
    let e = e.trim();
    if e.is_empty() {
        return Some("Enter your email");
    }
    let Some((a, d)) = e.split_once('@') else {
        return Some("Enter an email like name@company.com");
    };
    if a.is_empty() || d.contains('@') || e.contains(' ') {
        return Some("Enter an email like name@company.com");
    }
    match d.rsplit_once('.') {
        Some((h, t)) if !h.is_empty() && t.len() >= 2 => None,
        _ => Some("Enter an email like name@company.com"),
    }
}

/// Why a password is not valid, if it is not.
pub fn pass_error(p: &str) -> Option<&'static str> {
    if p.is_empty() {
        Some("Enter your password")
    } else if p.chars().count() < 8 {
        Some("Use at least 8 characters")
    } else {
        None
    }
}

/// The sign-in form.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SignIn {
    /// Email.
    pub email: String,
    /// Password.
    pub pass: String,
    /// Show the password.
    pub show: bool,
    /// Checks shown (after the first submit).
    pub touched: bool,
    /// Wrong tries.
    pub tries: u32,
    /// The form's error line.
    pub error: Option<String>,
    /// Signing in with: `""` none, `email`, or an SSO brand id.
    pub busy: &'static str,
    /// Signed in.
    pub done: bool,
}

impl SignIn {
    /// Fills the demo account.
    pub fn fill(&mut self) -> String {
        self.email = DEMO_EMAIL.into();
        self.pass = DEMO_PASS.into();
        self.error = None;
        "Demo account filled".into()
    }

    /// Whether the form is locked after three wrong tries.
    pub fn locked(&self) -> bool {
        self.tries >= 3
    }

    /// Submits: checks the fields, then the account.
    pub fn submit(&mut self) -> Result<String, String> {
        self.touched = true;
        if self.locked() {
            return Err("Too many tries. Reset your password to continue".into());
        }
        if let Some(e) = email_error(&self.email).or(pass_error(&self.pass)) {
            return Err(e.to_string());
        }
        if self.email.trim().eq_ignore_ascii_case(DEMO_EMAIL) && self.pass == DEMO_PASS {
            self.busy = "email";
            self.error = None;
            return Ok(format!("Signing in as {}", self.email.trim()));
        }
        self.tries += 1;
        let e = if self.locked() {
            "Too many tries. Reset your password to continue".to_string()
        } else {
            format!(
                "Email or password is wrong. {} {} left",
                3 - self.tries,
                if 3 - self.tries == 1 { "try" } else { "tries" }
            )
        };
        self.error = Some(e.clone());
        Err(e)
    }

    /// Starts SSO with provider `i`.
    pub fn sso(&mut self, i: usize) -> String {
        let (id, l) = SSO[i.min(SSO.len() - 1)];
        self.busy = id;
        self.error = None;
        format!("Continuing with {l}")
    }

    /// Finishes a sign-in in progress.
    pub fn finish(&mut self) -> Option<String> {
        if self.busy.is_empty() {
            return None;
        }
        self.busy = "";
        self.done = true;
        Some(format!("Signed in as {ME}"))
    }
}

// ───────────────────────────────────────────────────────────── live state ──

/// The live state of the five S14 screens a GPUI view keeps (a host routes press ids to
/// [`Arc3Ui::press`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arc3Ui {
    /// The market.
    pub mk: Market,
    /// The catalogue and cart.
    pub cat: Catalog,
    /// The invoices.
    pub iv: Invoice,
    /// The sign-in form.
    pub si: SignIn,
    /// The last thing done, said back.
    pub said: Option<String>,
}

impl Default for Arc3Ui {
    fn default() -> Self {
        let mut cat = Catalog::new(true);
        cat.open_id(1);
        Self {
            mk: Market::new(true),
            cat,
            iv: Invoice::new(true),
            si: SignIn::default(),
            said: None,
        }
    }
}

fn nth(s: &str, prefix: &str) -> Option<usize> {
    s.strip_prefix(prefix)?.parse().ok()
}

impl Arc3Ui {
    /// Handles a press id of the S14 screens; false when the id is not one of them. Ids:
    /// - market: `mk-sym-<n>`, `mk-range-<n>`, `mk-buy`, `mk-sell`, `mk-market`, `mk-limit`,
    ///   `mk-qty-up`, `mk-qty-down`, `mk-px-up`, `mk-px-down`, `mk-place`, `mk-cancel-<id>`,
    ///   `mk-tick`;
    /// - products: `pl-coll-<n>`, `pl-coll-all`, `pl-colour-<n>`, `pl-max`, `pl-sort`, `pl-view`,
    ///   `pl-clear`, `pl-open-<id>`, `pd-colour-<n>`, `pd-size-<n>`, `pd-qty-up`, `pd-qty-down`,
    ///   `pd-add`, `pd-back`;
    /// - invoice: `iv-open-<no>`, `iv-new`, `iv-add`, `iv-edit-<id>`, `iv-rm-<id>`, `iv-undo`,
    ///   `iv-qup-<id>`, `iv-qdown-<id>`, `iv-terms-<n>`, `iv-tax`, `iv-client-<n>`,
    ///   `iv-preview`, `iv-send`, `iv-remind`;
    /// - sign-in: `si-fill`, `si-show`, `si-submit`, `si-sso-<n>`, `si-finish`.
    pub fn press(&mut self, id: &str) -> bool {
        let out: Option<String> = if id.starts_with("mk-") {
            self.mk_press(id)
        } else if id.starts_with("pl-") || id.starts_with("pd-") {
            self.pl_press(id)
        } else if id.starts_with("iv-") {
            self.iv_press(id)
        } else if id.starts_with("si-") {
            self.si_press(id)
        } else {
            return false;
        };
        if let Some(s) = out {
            self.said = Some(s);
        }
        true
    }

    fn mk_press(&mut self, id: &str) -> Option<String> {
        let m = &mut self.mk;
        if let Some(n) = nth(id, "mk-sym-") {
            return Some(m.open_sym(n));
        }
        if let Some(n) = nth(id, "mk-range-") {
            m.range = n.min(MK_RANGES.len() - 1);
            return Some(MK_RANGES[m.range].0.to_string());
        }
        if let Some(n) = nth(id, "mk-cancel-") {
            return m.cancel(n as u32);
        }
        match id {
            "mk-buy" => {
                m.ticket.side = Side::Buy;
                Some("Buy".into())
            }
            "mk-sell" => {
                m.ticket.side = Side::Sell;
                Some("Sell".into())
            }
            "mk-market" => {
                m.ticket.limit = false;
                Some("Market order".into())
            }
            "mk-limit" => {
                m.ticket.limit = true;
                m.ticket.price = m.price();
                Some("Limit order".into())
            }
            "mk-qty-up" | "mk-qty-down" => {
                let d: i64 = if id == "mk-qty-up" { 1 } else { -1 };
                m.ticket.qty = (m.ticket.qty as i64 + d).clamp(0, 9_999) as u32;
                Some(format!("{} shares", m.ticket.qty))
            }
            "mk-px-up" | "mk-px-down" => {
                let d = if id == "mk-px-up" { 5 } else { -5 };
                m.ticket.price = (m.ticket.price + d).max(1);
                Some(format!("Limit {}", money(m.ticket.price)))
            }
            "mk-place" => Some(match m.place() {
                Ok(s) | Err(s) => s,
            }),
            "mk-tick" => {
                m.step();
                None
            }
            _ => None,
        }
    }

    fn pl_press(&mut self, id: &str) -> Option<String> {
        let c = &mut self.cat;
        if let Some(n) = nth(id, "pl-coll-") {
            let n = n.min(COLLS.len() - 1);
            c.coll = if c.coll == Some(n) { None } else { Some(n) };
            return Some(format!("{} · {} products", COLLS[n].0, c.shown().len()));
        }
        if let Some(n) = nth(id, "pl-colour-") {
            let n = n.min(COLOURS.len() - 1);
            c.colour = if c.colour == Some(n) { None } else { Some(n) };
            return Some(format!("{} products", c.shown().len()));
        }
        if let Some(n) = nth(id, "pl-open-") {
            return c.open_id(n as u32);
        }
        if let Some(n) = nth(id, "pd-colour-") {
            let ok = c.current().is_some_and(|p| p.colours.contains(&n));
            if ok {
                c.pick_colour = n;
                return Some(COLOURS[n].0.to_string());
            }
            return None;
        }
        if let Some(n) = nth(id, "pd-size-") {
            return Some(c.size(n));
        }
        match id {
            "pl-coll-all" => {
                c.coll = None;
                Some("All products".into())
            }
            "pl-max" => {
                c.max = if c.max.is_some() { None } else { Some(15_000) };
                Some(if c.max.is_some() {
                    "Under $150".into()
                } else {
                    "Any price".into()
                })
            }
            "pl-sort" => {
                c.sort = (c.sort + 1) % SORTS.len();
                Some(format!("Sorted: {}", SORTS[c.sort].1))
            }
            "pl-view" => {
                c.list = !c.list;
                Some(if c.list { "List view" } else { "Grid view" }.into())
            }
            "pl-clear" => Some(c.clear()),
            "pd-qty-up" => Some(format!("Quantity {}", c.step_qty(1))),
            "pd-qty-down" => Some(format!("Quantity {}", c.step_qty(-1))),
            "pd-add" => Some(match c.add() {
                Ok(s) | Err(s) => s,
            }),
            "pd-back" => {
                c.open = None;
                Some("Products".into())
            }
            _ => None,
        }
    }

    fn iv_press(&mut self, id: &str) -> Option<String> {
        let v = &mut self.iv;
        if let Some(n) = nth(id, "iv-open-") {
            return v.open_no(n as u32);
        }
        if let Some(n) = nth(id, "iv-edit-") {
            if v.editable() {
                v.edit = if v.edit == Some(n as u32) {
                    None
                } else {
                    Some(n as u32)
                };
            }
            return None;
        }
        if let Some(n) = nth(id, "iv-rm-") {
            return v.remove_line(n as u32);
        }
        if let Some(n) = nth(id, "iv-qup-").or_else(|| nth(id, "iv-qdown-")) {
            let d: i64 = if id.starts_with("iv-qup-") { 1 } else { -1 };
            let q = v.current()?.lines.iter().find(|l| l.id == n as u32)?.qty;
            let q = (q as i64 + d).max(0) as u32;
            v.set_line(n as u32, None, Some(q), None);
            return Some(format!("Quantity {q}"));
        }
        if let Some(n) = nth(id, "iv-terms-") {
            return v.terms(n);
        }
        if let Some(n) = nth(id, "iv-client-") {
            return v.client(n);
        }
        match id {
            "iv-new" => Some(v.new_draft()),
            "iv-add" => v.add_line(),
            "iv-undo" => v.undo(),
            "iv-tax" => v.toggle_tax(),
            "iv-preview" => {
                v.preview = !v.preview;
                v.edit = None;
                Some(if v.preview { "Preview" } else { "Editing" }.into())
            }
            "iv-send" => Some(match v.send() {
                Ok(s) | Err(s) => s,
            }),
            "iv-remind" => {
                v.remind = !v.remind;
                Some(
                    if v.remind {
                        "Reminders on"
                    } else {
                        "Reminders off"
                    }
                    .into(),
                )
            }
            _ => None,
        }
    }

    fn si_press(&mut self, id: &str) -> Option<String> {
        let s = &mut self.si;
        if let Some(n) = nth(id, "si-sso-") {
            return Some(s.sso(n));
        }
        match id {
            "si-fill" => Some(s.fill()),
            "si-show" => {
                s.show = !s.show;
                Some(
                    if s.show {
                        "Password shown"
                    } else {
                        "Password hidden"
                    }
                    .into(),
                )
            }
            "si-submit" => Some(match s.submit() {
                Ok(x) | Err(x) => x,
            }),
            "si-finish" => s.finish(),
            _ => None,
        }
    }
}

/// Static press ids for numbered presses (GPUI ids are `&'static str`).
pub mod ids {
    pub use super::super::arcscr::ids::at;
    macro_rules! table {
        ($(#[$m:meta])* $name:ident, $p:literal, $($n:literal),*) => {
            $(#[$m])*
            pub const $name: &[&str] = &[$(concat!($p, $n)),*];
        };
    }
    table!(
        /// `mk-sym-0` … `mk-sym-6`.
        MK_SYM, "mk-sym-", 0, 1, 2, 3, 4, 5, 6
    );
    table!(
        /// `mk-range-0` … `mk-range-4`.
        MK_RANGE, "mk-range-", 0, 1, 2, 3, 4
    );
    table!(
        /// `mk-cancel-0` … `mk-cancel-15` (by order id).
        MK_CANCEL, "mk-cancel-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
    );
    table!(
        /// `pl-coll-0` … `pl-coll-3`.
        PL_COLL, "pl-coll-", 0, 1, 2, 3
    );
    table!(
        /// `pl-colour-0` … `pl-colour-5`.
        PL_COLOUR, "pl-colour-", 0, 1, 2, 3, 4, 5
    );
    table!(
        /// `pl-open-0` … `pl-open-10` (by product id).
        PL_OPEN, "pl-open-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
    );
    table!(
        /// `pd-colour-0` … `pd-colour-5`.
        PD_COLOUR, "pd-colour-", 0, 1, 2, 3, 4, 5
    );
    table!(
        /// `pd-size-0` … `pd-size-4`.
        PD_SIZE, "pd-size-", 0, 1, 2, 3, 4
    );
    table!(
        /// `iv-open-1040` … `iv-open-1050` (by number).
        IV_OPEN, "iv-open-", 1040, 1041, 1042, 1043, 1044, 1045, 1046, 1047, 1048, 1049, 1050
    );
    table!(
        /// `iv-edit-0` … `iv-edit-9` (by line id).
        IV_EDIT, "iv-edit-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
    );
    table!(
        /// `iv-rm-0` … `iv-rm-9` (by line id).
        IV_RM, "iv-rm-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
    );
    table!(
        /// `iv-qup-0` … `iv-qup-9` (by line id).
        IV_QUP, "iv-qup-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
    );
    table!(
        /// `iv-qdown-0` … `iv-qdown-9` (by line id).
        IV_QDOWN, "iv-qdown-", 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
    );
    table!(
        /// `iv-terms-0` … `iv-terms-3`.
        IV_TERMS, "iv-terms-", 0, 1, 2, 3
    );
    table!(
        /// `iv-client-0` … `iv-client-3`.
        IV_CLIENT, "iv-client-", 0, 1, 2, 3
    );
    table!(
        /// `si-sso-0` … `si-sso-1`.
        SI_SSO, "si-sso-", 0, 1
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s14_money_and_states() {
        assert_eq!(money(1_234_567), "$12,345.67");
        assert_eq!(money(-505), "-$5.05");
        assert_eq!(money0(1_234_567), "$12,346");
        assert_eq!(pct(127), "+1.27%");
        assert_eq!(pct(-5), "-0.05%");
        assert_eq!(s14_states("invoice-studio").len(), 6);
        assert_eq!(arc3_split("wrong").1, "wrong");
        assert_eq!(arc3_split("empty").0, ScreenState::Empty);
    }

    #[test]
    fn s14_market_series_ticket() {
        for (s, sym) in SYMBOLS.iter().enumerate() {
            for (r, rg) in MK_RANGES.iter().enumerate() {
                let v = series(s, r);
                assert_eq!(v.len(), rg.1);
                assert_eq!(*v.last().unwrap(), sym.2);
                assert!(v.iter().all(|x| *x > 0));
            }
        }
        assert_eq!(series(2, 3), series(2, 3), "deterministic");
        assert_eq!(live_price(0, 0), SYMBOLS[0].2);
        assert_ne!(live_price(0, 1), live_price(0, 2));
        let mut m = Market::new(true);
        let v0 = m.value();
        m.ticket.qty = 0;
        assert_eq!(m.place(), Err("Enter a number of shares".into()));
        m.ticket.qty = 10;
        let cash = m.cash;
        let r = m.place().unwrap();
        assert!(r.starts_with("Filled: Buy 10 KSTR at $186.42"), "{r}");
        assert_eq!(m.held[0], 50);
        assert_eq!(m.cash, cash - 10 * 18_642);
        assert_eq!(m.value(), v0);
        m.ticket.qty = 9_000;
        assert!(m.check().unwrap().starts_with("Not enough cash"));
        m.ticket.side = Side::Sell;
        m.ticket.qty = 60;
        assert_eq!(m.check().as_deref(), Some("You hold 50 KSTR"));
        m.ticket.qty = 50;
        m.place().unwrap();
        assert_eq!(m.held[0], 0);
        assert_eq!(m.cost[0], 0);
        m.open_sym(1);
        m.ticket.side = Side::Buy;
        m.ticket.limit = true;
        m.ticket.price = 9_000;
        m.ticket.qty = 5;
        assert_eq!(
            m.place().as_deref(),
            Ok("Limit order placed: Buy 5 NVLA at $90.00")
        );
        assert_eq!(m.orders.len(), 3);
        assert_eq!(m.cancel(1).as_deref(), Some("Cancelled 25 NVLA"));
        assert!(m.cancel(1).is_none());
        m.open = false;
        m.step();
        assert_eq!(m.tick, 0, "closed market does not tick");
        assert!(Market::new(false).positions().is_empty());
    }

    #[test]
    fn s14_catalog_filter_sort_cart() {
        let mut c = Catalog::new(true);
        assert_eq!(c.shown().len(), 10);
        c.coll = Some(0);
        assert_eq!(c.shown().len(), 3);
        c.colour = Some(3);
        assert_eq!(c.shown().iter().map(|p| p.id).collect::<Vec<_>>(), vec![3]);
        c.clear();
        c.sort = 1;
        assert_eq!(c.shown()[0].id, 6, "cheapest first");
        c.sort = 2;
        assert_eq!(c.shown()[0].id, 1);
        c.sort = 3;
        assert_eq!(c.shown()[0].id, 6, "newest first");
        c.max = Some(15_000);
        assert!(c.shown().iter().all(|p| p.price <= 15_000));
        c.query = "zzz".into();
        assert!(c.shown().is_empty());
        c.clear();
        c.open_id(3);
        assert_eq!(c.add(), Err("Pick a size".into()));
        assert_eq!(c.size(0), "XS is sold out");
        assert_eq!(c.pick_size, None);
        assert_eq!(c.size(2), "Size M: only 3 left");
        assert_eq!(c.step_qty(5), 3);
        assert_eq!(c.step_qty(-9), 1);
        c.qty = 2;
        assert_eq!(
            c.add().as_deref(),
            Ok("Added 2 × Ridgeway Puffer, Charcoal M")
        );
        c.qty = 2;
        assert_eq!(c.add(), Err("Only 3 in M".into()));
        assert_eq!(c.cart_count(), 2);
        assert_eq!(c.cart_total(), 44_800);
        assert_eq!(stock_word(2), ("2 left".into(), "amber"));
    }

    #[test]
    fn s14_invoice_lines_totals_send() {
        let mut v = Invoice::new(true);
        let i = v.current().unwrap();
        assert_eq!(i.subtotal(), 360_000 + 175_000 + 34_900);
        assert_eq!(i.tax_amount(), 47_017);
        assert_eq!(i.total(), 616_917);
        assert_eq!(iso(i.due()), "2026-12-12");
        assert_eq!(v.terms(1).as_deref(), Some("Net 15 · due 2026-11-27"));
        v.add_line();
        assert_eq!(v.edit, Some(4));
        assert_eq!(v.check(), Some("Describe every line"));
        assert!(v.set_line(4, Some("Rush fee"), Some(1), Some(25_000)));
        assert_eq!(v.current().unwrap().subtotal(), 594_900);
        assert_eq!(v.remove_line(2).as_deref(), Some("Removed Logo refinement"));
        assert_eq!(v.current().unwrap().lines.len(), 3);
        assert_eq!(v.undo().as_deref(), Some("Line restored"));
        assert_eq!(v.current().unwrap().lines[1].id, 2);
        assert_eq!(v.toggle_tax().as_deref(), Some("Sales tax off"));
        assert_eq!(v.current().unwrap().tax_amount(), 0);
        v.client(3);
        assert_eq!(v.check(), Some("Add the client's billing email"));
        v.client(0);
        let s = v.send().unwrap();
        assert_eq!(
            s,
            "NS-1046 sent to accounts@larkspur.example · $5,949.00 due 2026-11-27"
        );
        assert!(!v.editable());
        assert!(v.add_line().is_none());
        assert_eq!(v.send(), Err("Already sent".into()));
        assert_eq!(v.count(1), 2);
        assert!(v.outstanding() > 0);
        assert_eq!(v.new_draft(), "Draft NS-1047");
        assert_eq!(v.check(), Some("Add a line item"));
    }

    #[test]
    fn s14_sign_in_checks() {
        assert_eq!(email_error(""), Some("Enter your email"));
        assert!(email_error("rowan@").is_some());
        assert!(email_error("rowan@x").is_some());
        assert!(email_error("ro wan@x.io").is_some());
        assert_eq!(email_error("rowan@x.io"), None);
        assert_eq!(pass_error("short"), Some("Use at least 8 characters"));
        let mut s = SignIn::default();
        assert_eq!(s.submit(), Err("Enter your email".into()));
        s.email = DEMO_EMAIL.into();
        s.pass = "wrong-password".into();
        assert_eq!(
            s.submit(),
            Err("Email or password is wrong. 2 tries left".into())
        );
        s.submit().unwrap_err();
        assert_eq!(
            s.submit(),
            Err("Too many tries. Reset your password to continue".into())
        );
        assert!(s.locked());
        let mut s = SignIn::default();
        s.fill();
        assert!(s.submit().unwrap().starts_with("Signing in as"));
        assert_eq!(s.finish().as_deref(), Some("Signed in as Rowan Vale"));
        assert_eq!(s.sso(1), "Continuing with GitHub");
    }

    #[test]
    fn s14_live_presses() {
        let mut u = Arc3Ui::default();
        assert!(u.press("mk-sym-2"));
        assert_eq!(u.mk.sym, 2);
        assert!(u.press("mk-range-4"));
        assert!(u.press("mk-tick"));
        assert_eq!(u.mk.tick, 1);
        assert!(u.press("mk-limit"));
        assert!(u.press("mk-place"));
        assert!(u.said.as_deref().unwrap().starts_with("Limit order placed"));
        assert!(u.press("pl-coll-2"));
        assert_eq!(u.cat.shown().len(), 2);
        assert!(u.press("pl-open-8"));
        assert!(u.press("pd-size-2"));
        assert!(u.press("pd-add"));
        assert_eq!(u.cat.cart_count(), 1);
        assert!(u.press("iv-add"));
        assert!(u.press("iv-qup-4"));
        assert!(u.press("iv-rm-4"));
        assert!(u.press("iv-undo"));
        assert_eq!(u.iv.current().unwrap().lines.len(), 4);
        assert!(u.press("si-submit"));
        assert_eq!(u.said.as_deref(), Some("Enter your email"));
        assert!(u.press("si-fill"));
        assert!(u.press("si-submit"));
        assert!(u.press("si-finish"));
        assert!(u.si.done);
        assert!(!u.press("cal-view-2"), "not an S14 press");
    }
}
