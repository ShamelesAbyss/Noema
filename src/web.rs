use crate::lexicon::LexiconEngine;
use ::rand::Rng;
use macroquad::prelude::{
    draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines, draw_text,
    draw_triangle, Color, Vec2,
};
use serde::Deserialize;
use std::collections::HashSet;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WordStyle {
    ArticleTitle,
    SectionHeader,
    Hatnote,
    BodyText,
    SemanticAccent,
    Hyperlink,
    Citation,
    ListMarker,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WordState {
    InPage,
    Tethered { crawler_id: usize, limb_idx: usize },
    Plucked { timer: f32, max_life: f32, filled_highlight: bool },
    Vaporized { timer: f32, respawn_time: f32 },
}

#[derive(Clone, Debug)]
pub struct WordToken {
    pub text: String,
    pub clean_word: String,
    pub style: WordStyle,
    pub state: WordState,
    pub home_pos: Vec2,
    pub pos: Vec2,
    pub vel: Vec2,
    pub angle: f32,
    pub ang_vel: f32,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub shadow_carve: f32,
    pub mutate_flash: f32,
    pub heat: f32,
    pub box_excite: f32,
    pub box_filled: bool,
    pub tether_anchor: Option<Vec2>,
    pub chip_rgb: (f32, f32, f32),
    pub link_target: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct PageParticle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: (f32, f32, f32),
}

#[derive(Clone, Copy, Debug)]
pub struct PageShockwave {
    pub pos: Vec2,
    pub radius: f32,
    pub max_radius: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: (f32, f32, f32),
}

#[derive(Clone, Debug)]
pub struct InfoboxClickable {
    pub rect_min: Vec2,
    pub rect_max: Vec2,
    pub target_topic: String,
}

pub struct InfoboxWidget {
    pub pos: Vec2,
    pub width: f32,
    pub height: f32,
    pub title: String,
    pub taxonomy_rows: Vec<(String, String)>,
    pub sub_taxa: Vec<String>,
    pub diversity_stat: String,
    pub clickables: Vec<InfoboxClickable>,
}

pub struct InputBoxWidget {
    pub pos: Vec2,
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize)]
struct MediaWikiResponse {
    query: Option<MediaWikiQuery>,
}

#[derive(Deserialize)]
struct MediaWikiQuery {
    pages: Option<Vec<MediaWikiPage>>,
    search: Option<Vec<MediaWikiSearchHit>>,
}

#[derive(Deserialize)]
struct MediaWikiSearchHit {
    title: Option<String>,
}

#[derive(Deserialize)]
struct MediaWikiPage {
    title: Option<String>,
    extract: Option<String>,
    links: Option<Vec<MediaWikiLink>>,
}

#[derive(Deserialize)]
struct MediaWikiLink {
    title: Option<String>,
}

pub struct FetchedWikiPage {
    pub title: String,
    pub extract: String,
    pub linked_titles: Vec<String>,
}

pub struct WikiBridge {
    req_tx: Sender<String>,
    res_rx: Receiver<Result<FetchedWikiPage, String>>,
    pub last_status: String,
}

impl WikiBridge {
    pub fn new() -> Self {
        let (req_tx, req_rx) = mpsc::channel::<String>();
        let (res_tx, res_rx) = mpsc::channel::<Result<FetchedWikiPage, String>>();

        thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout(std::time::Duration::from_secs(8))
                .user_agent("Noema/0.8 (Autonomous Wikipedia Node Explorer)")
                .build();

            let fetch_by_title = |target: &str| -> Option<FetchedWikiPage> {
                let encoded = target
                    .trim()
                    .replace(' ', "_")
                    .replace('&', "%26")
                    .replace('?', "%3F");
                let url = format!(
                    "https://en.wikipedia.org/w/api.php?action=query&prop=extracts|links&explaintext=1&pllimit=250&redirects=1&format=json&formatversion=2&titles={}",
                    encoded
                );
                if let Ok(resp) = agent.get(&url).call() {
                    if let Ok(parsed) = resp.into_json::<MediaWikiResponse>() {
                        if let Some(page) = parsed
                            .query
                            .and_then(|q| q.pages)
                            .and_then(|mut p| p.pop())
                        {
                            let title = page.title.unwrap_or_else(|| target.to_string());
                            let extract = page.extract.unwrap_or_default();
                            let linked_titles: Vec<String> = page
                                .links
                                .unwrap_or_default()
                                .into_iter()
                                .filter_map(|l| l.title)
                                .filter(|t| !t.contains(':'))
                                .collect();
                            if extract.len() > 60 {
                                return Some(FetchedWikiPage {
                                    title,
                                    extract,
                                    linked_titles,
                                });
                            }
                        }
                    }
                }
                None
            };

            while let Ok(topic) = req_rx.recv() {
                if let Some(page) = fetch_by_title(&topic) {
                    let _ = res_tx.send(Ok(page));
                    continue;
                }

                // Fallback: Search Wikipedia so ANY clicked highlighted word routes to a real article
                let encoded_search = topic
                    .trim()
                    .replace(' ', "+")
                    .replace('&', "%26")
                    .replace('?', "%3F");
                let search_url = format!(
                    "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&srlimit=1&format=json&formatversion=2",
                    encoded_search
                );
                if let Ok(resp) = agent.get(&search_url).call() {
                    if let Ok(parsed) = resp.into_json::<MediaWikiResponse>() {
                        if let Some(best_title) = parsed
                            .query
                            .and_then(|q| q.search)
                            .and_then(|mut s| s.pop())
                            .and_then(|h| h.title)
                        {
                            if let Some(page) = fetch_by_title(&best_title) {
                                let _ = res_tx.send(Ok(page));
                            }
                        }
                    }
                }
            }
        });

        Self {
            req_tx,
            res_rx,
            last_status: "Click ANY highlighted word, link, chip, or infobox taxon to route to its Wikipedia instance".to_string(),
        }
    }

    pub fn request_topic(&mut self, topic: &str) {
        let cleaned = topic
            .trim()
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != ' ');
        if !cleaned.is_empty() {
            self.last_status = format!("Routing to Wikipedia instance: '{}'...", cleaned);
            let _ = self.req_tx.send(cleaned.to_string());
        }
    }
}

pub fn open_url_in_browser(title: &str) {
    let encoded = title.trim().replace(' ', "_");
    let url = format!("https://en.wikipedia.org/wiki/{}", encoded);
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("cmd").args(["/C", "start", "", &url]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(&url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(&url).spawn();
    }
}

pub struct KnowledgeWeb {
    pub current_title: String,
    pub words: Vec<WordToken>,
    pub divider_lines: Vec<(Vec2, Vec2)>,
    pub particles: Vec<PageParticle>,
    pub shockwaves: Vec<PageShockwave>,
    pub infobox: InfoboxWidget,
    pub synthesis_box: InputBoxWidget,
    pub cladogram_pos: Vec2,
    pub cladogram_taxa: Vec<String>,
    pub preset_index: usize,
    pub bridge: WikiBridge,
    pub ui_phase: f32,
}

const FEATURED_TOPICS: [&str; 8] = [
    "Centipede",
    "Spider",
    "Insect",
    "Cephalopod",
    "Arthropod",
    "Bioluminescence",
    "Mycorrhizal_network",
    "Cybernetics",
];

impl KnowledgeWeb {
    pub fn new(lexicon: &mut LexiconEngine) -> Self {
        let mut web = Self {
            current_title: "Centipede".to_string(),
            words: Vec::new(),
            divider_lines: Vec::new(),
            particles: Vec::with_capacity(350),
            shockwaves: Vec::with_capacity(40),
            infobox: InfoboxWidget {
                pos: Vec2::new(260.0, -450.0),
                width: 315.0,
                height: 610.0,
                title: "Chilopoda".to_string(),
                taxonomy_rows: Vec::new(),
                sub_taxa: Vec::new(),
                diversity_stat: "139 families, c. 53,700 species".to_string(),
                clickables: Vec::new(),
            },
            synthesis_box: InputBoxWidget {
                pos: Vec2::new(-530.0, 220.0),
                width: 755.0,
                height: 36.0,
            },
            cladogram_pos: Vec2::new(-530.0, 420.0),
            cladogram_taxa: vec![
                "Notostigmophora".to_string(),
                "Scutigeromorpha".to_string(),
                "Pleurostigmophora".to_string(),
            ],
            preset_index: 0,
            bridge: WikiBridge::new(),
            ui_phase: 0.0,
        };

        web.load_full_wiki_text(
            "Centipede",
            "It was previously believed to create segments and 15 pairs of legs so there are adapted to their tendency to bee stings. For example, one following lineage:\n1. A spiral ridge occurs on second maxilliped has an egg tooth.\n2. The first postcephalic appendage is modified into forcipules.\n3. The first postcephalic appendage is Geophilomorpha to venom claws.\n4. A spiral ridge occurs on second maxilliped 75 believed egg generalist.\nJust as it is the legs (the tail, you will be an honest signal of centipedes. Rhysida immarginata togoensis makes a double segment bears a single pair of legs may play a role in high-humidity environments as shown in the Serpent Island centipede. Like insects, bristles and tail, which, with the first body segment, toxic secretions contain hydrogen cyanide and segments are about five stages) and the position of the Rhynie chert, Rhyniognatha, Geophilomorpha and even bats, overlap, they of predominantly generalist predators.\n• The head is somewhat flatter.\n• The centipedes can develop through either anamorphosis or epimorphosis.\n• The head is somewhat flatter in subterranean species.\n• The centipedes can develop through either anamorphosis or scolopendra.\nIt was previously believed that Chilopoda was split into Anamorpha (Lithobiomorpha and Scutigeromorpha) and Epimorpha (Geophilomorpha and Scolopendromorpha), based on developmental modes, with the relationship of the Craterostigmomorpha being uncertain. Recent phylogenetic analyses using combined molecular and morphological characters supports the previous phylogeny. The Epimorpha still exist as a monophyletic group within the Pleurostigmophora, but the Anamorpha are paraphyletic, as shown in the cladogram:",
            vec![
                "Arthropod".to_string(),
                "Myriapoda".to_string(),
                "Chilopoda".to_string(),
                "Scutigeromorpha".to_string(),
                "Geophilomorpha".to_string(),
                "Scolopendromorpha".to_string(),
                "Lithobiomorpha".to_string(),
                "Forcipules".to_string(),
                "Monophyletic".to_string(),
                "Paraphyletic".to_string(),
                "Cladogram".to_string(),
            ],
            lexicon,
        );
        web.bridge.request_topic(FEATURED_TOPICS[0]);
        web
    }

    pub fn spawn_burst(&mut self, pos: Vec2, color: (f32, f32, f32), count: usize, speed: f32) {
        if self.particles.len() > 500 {
            return;
        }
        let mut rng = ::rand::thread_rng();
        for _ in 0..count {
            let ang = rng.gen_range(0.0_f32..std::f32::consts::TAU);
            let spd = rng.gen_range(speed * 0.28..speed * 1.15);
            self.particles.push(PageParticle {
                pos,
                vel: Vec2::new(ang.cos() * spd, ang.sin() * spd),
                life: 0.0,
                max_life: rng.gen_range(0.35..1.15),
                size: rng.gen_range(1.6..3.8),
                color,
            });
        }
    }

    pub fn spawn_shockwave(&mut self, pos: Vec2, max_radius: f32, color: (f32, f32, f32)) {
        if self.shockwaves.len() < 28 {
            self.shockwaves.push(PageShockwave {
                pos,
                radius: 8.0,
                max_radius,
                life: 0.0,
                max_life: 0.55,
                color,
            });
        }
    }

    pub fn cycle_preset_article(&mut self, _lexicon: &mut LexiconEngine) {
        self.preset_index = (self.preset_index + 1) % FEATURED_TOPICS.len();
        self.bridge.request_topic(FEATURED_TOPICS[self.preset_index]);
    }

    fn populate_infobox_metadata(&mut self, title: &str, linked_titles: &[String]) {
        self.infobox.title = title.to_string();

        // Build clean, zero-overflow 2-column rows inspired by Wikipedia's Scientific classification box
        let mut links_iter = linked_titles
            .iter()
            .filter(|l| l.len() >= 4 && l.len() <= 20);
        let k1 = links_iter
            .next()
            .cloned()
            .unwrap_or_else(|| "Biota".to_string());
        let k2 = links_iter
            .next()
            .cloned()
            .unwrap_or_else(|| "Arthropoda".to_string());
        let k3 = links_iter
            .next()
            .cloned()
            .unwrap_or_else(|| "Tetrapulmonata".to_string());
        let k4 = links_iter
            .next()
            .cloned()
            .unwrap_or_else(|| "Chilopoda".to_string());
        let k5 = links_iter
            .next()
            .cloned()
            .unwrap_or_else(|| title.to_string());

        self.infobox.taxonomy_rows = vec![
            ("Kingdom:".to_string(), truncate_str(&k1, 16)),
            ("Phylum:".to_string(), truncate_str(&k2, 16)),
            ("Subphylum:".to_string(), truncate_str(&k3, 16)),
            ("Class:".to_string(), truncate_str(&k4, 16)),
            ("Order:".to_string(), truncate_str(&k5, 16)),
            ("Authority:".to_string(), "Clerck, 1757".to_string()),
        ];

        self.infobox.sub_taxa = linked_titles
            .iter()
            .filter(|s| s.len() >= 5 && s.len() <= 22)
            .take(4)
            .cloned()
            .collect();

        if self.infobox.sub_taxa.is_empty() {
            self.infobox.sub_taxa = vec![
                "Mesothelae".to_string(),
                "Araneomorphae".to_string(),
                "Mygalomorphae".to_string(),
                "Pleurostigmophora".to_string(),
            ];
        }

        self.cladogram_taxa = linked_titles
            .iter()
            .filter(|s| s.len() >= 6 && s.len() <= 20)
            .skip(2)
            .take(3)
            .cloned()
            .collect();
        while self.cladogram_taxa.len() < 3 {
            self.cladogram_taxa.push("Scutigeromorpha".to_string());
        }

        let fam_count = 40 + (title.len() * 17) % 160;
        let sp_count = 1_200 + (linked_titles.len() * 843) % 65_000;
        self.infobox.diversity_stat = format!("{} families, c. {} species", fam_count, sp_count);
    }

    pub fn load_full_wiki_text(
        &mut self,
        title: &str,
        extract: &str,
        linked_titles: Vec<String>,
        lexicon: &mut LexiconEngine,
    ) {
        self.words.clear();
        self.divider_lines.clear();
        self.current_title = title.to_string();
        lexicon.ingest_corpus(extract);
        self.populate_infobox_metadata(title, &linked_titles);

        let link_lookup: HashSet<String> =
            linked_titles.iter().map(|s| s.to_lowercase()).collect();

        // Main article column sits cleanly on the left; Infobox sits on the right without collision
        let left_margin = -530.0_f32;
        let max_width = 755.0_f32;
        let mut y = -440.0_f32;

        // Primary Title
        self.push_line_tokens(
            title,
            left_margin,
            &mut y,
            max_width,
            WordStyle::ArticleTitle,
        );
        self.divider_lines.push((
            Vec2::new(left_margin, y - 4.0),
            Vec2::new(left_margin + max_width, y - 4.0),
        ));
        y += 18.0;

        let mut placed_synthesis_box = false;
        let mut paragraph_count = 0usize;
        let mut list_counter = 1usize;

        for raw_line in extract.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            if self.words.len() > 2300 {
                break;
            }

            if line.starts_with("==") && line.ends_with("==") {
                let clean = line.trim_matches('=').trim();
                y += 18.0;
                self.push_line_tokens(
                    clean,
                    left_margin,
                    &mut y,
                    max_width,
                    WordStyle::SectionHeader,
                );
                self.divider_lines.push((
                    Vec2::new(left_margin, y - 2.0),
                    Vec2::new(left_margin + max_width, y - 2.0),
                ));
                y += 10.0;

                // Authentic Wikipedia "Main article: ..." hatnote under sections
                if let Some(hat_topic) = linked_titles.get(paragraph_count % linked_titles.len().max(1)) {
                    self.push_hatnote(hat_topic, left_margin, &mut y);
                    y += 8.0;
                }
                list_counter = 1;
            } else if line.starts_with("•")
                || line.starts_with("* ")
                || line.starts_with("- ")
                || (line.len() > 2
                    && line.chars().next().unwrap().is_ascii_digit()
                    && line.chars().nth(1) == Some('.'))
            {
                self.push_list_item(line, left_margin + 26.0, &mut y, max_width - 36.0, &link_lookup);
            } else {
                // Inject structured numbered/bulleted rows periodically for long plain-text extracts
                if paragraph_count == 1 && list_counter <= 4 && line.len() > 140 {
                    for sentence in line.split(". ").take(4) {
                        if !sentence.trim().is_empty() {
                            let numbered = format!("{}. {}.", list_counter, sentence.trim().trim_end_matches('.'));
                            self.push_list_item(
                                &numbered,
                                left_margin + 28.0,
                                &mut y,
                                max_width - 40.0,
                                &link_lookup,
                            );
                            list_counter += 1;
                        }
                    }
                    y += 10.0;
                } else {
                    self.push_full_wiki_paragraph(
                        line,
                        left_margin,
                        &mut y,
                        max_width,
                        &link_lookup,
                        paragraph_count,
                    );
                    y += 14.0;
                }

                paragraph_count += 1;
                if paragraph_count == 1 && !placed_synthesis_box {
                    y += 4.0;
                    self.synthesis_box.pos = Vec2::new(left_margin, y);
                    y += 52.0;
                    placed_synthesis_box = true;
                }
            }
        }

        if !placed_synthesis_box {
            self.synthesis_box.pos = Vec2::new(left_margin, y);
            y += 52.0;
        }

        // Position the phylogenetic cladogram widget right below the main text
        y += 16.0;
        self.cladogram_pos = Vec2::new(left_margin, y);
        y += 95.0;

        if !linked_titles.is_empty() {
            y += 12.0;
            self.push_line_tokens(
                "See also",
                left_margin,
                &mut y,
                max_width,
                WordStyle::SectionHeader,
            );
            self.divider_lines.push((
                Vec2::new(left_margin, y - 2.0),
                Vec2::new(left_margin + max_width, y - 2.0),
            ));
            y += 14.0;

            let mut x = left_margin;
            for link_title in linked_titles.iter().take(28) {
                let est_w = (link_title.len() as f32 * 18.0 * 0.49) + 12.0;
                if x + est_w > left_margin + max_width && x > left_margin {
                    x = left_margin;
                    y += 28.0;
                }
                let w = self.push_single_token(
                    link_title,
                    Vec2::new(x, y),
                    18.0,
                    WordStyle::Hyperlink,
                    Some(link_title.replace(' ', "_")),
                );
                x += w + 22.0;
            }
        }
    }

    pub fn load_full_wiki_article(&mut self, page: FetchedWikiPage, lexicon: &mut LexiconEngine) {
        self.load_full_wiki_text(&page.title, &page.extract, page.linked_titles, lexicon);
    }

    fn push_hatnote(&mut self, topic: &str, start_x: f32, y: &mut f32) {
        let fs = 17.5_f32;
        let w1 = self.push_single_token("Main", Vec2::new(start_x, *y), fs, WordStyle::Hatnote, None);
        let w2 = self.push_single_token(
            "article:",
            Vec2::new(start_x + w1 + 7.0, *y),
            fs,
            WordStyle::Hatnote,
            None,
        );
        let _ = self.push_single_token(
            topic,
            Vec2::new(start_x + w1 + w2 + 16.0, *y),
            fs,
            WordStyle::Hyperlink,
            Some(topic.replace(' ', "_")),
        );
        *y += 26.0;
    }

    fn push_list_item(
        &mut self,
        text: &str,
        start_x: f32,
        y: &mut f32,
        max_width: f32,
        link_lookup: &HashSet<String>,
    ) {
        let font_size = 18.5_f32;
        let line_height = 26.0_f32;
        let mut x = start_x;

        for (i, raw) in text.split_whitespace().enumerate() {
            let clean = raw
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                .to_string();
            let lower = clean.to_lowercase();

            let (style, link) = if i == 0 && (raw == "•" || raw.ends_with('.')) {
                (WordStyle::ListMarker, None)
            } else if clean.len() >= 4 && link_lookup.contains(&lower) {
                (WordStyle::Hyperlink, Some(clean.clone()))
            } else if clean.len() >= 7 && i % 5 == 2 {
                (WordStyle::SemanticAccent, Some(clean.clone()))
            } else if clean.len() >= 4 {
                (WordStyle::BodyText, Some(clean.clone()))
            } else {
                (WordStyle::BodyText, None)
            };

            let est_w = (raw.len() as f32 * font_size * 0.49) + 4.0;
            if x + est_w > start_x + max_width && x > start_x {
                x = start_x + 22.0;
                *y += line_height;
            }
            let w = self.push_single_token(raw, Vec2::new(x, *y), font_size, style, link);
            x += w + 7.5;
        }
        *y += line_height + 2.0;
    }

    fn push_line_tokens(
        &mut self,
        text: &str,
        start_x: f32,
        y: &mut f32,
        max_width: f32,
        style: WordStyle,
    ) {
        let font_size = if style == WordStyle::ArticleTitle {
            28.0_f32
        } else {
            24.0_f32
        };
        let mut x = start_x;
        for raw in text.split_whitespace() {
            let est_w = (raw.len() as f32 * font_size * 0.50) + 6.0;
            if x + est_w > start_x + max_width && x > start_x {
                x = start_x;
                *y += font_size * 1.35;
            }
            let clean = raw
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                .to_string();
            let link = if clean.len() >= 4 {
                Some(clean)
            } else {
                None
            };
            let w = self.push_single_token(raw, Vec2::new(x, *y), font_size, style, link);
            x += w + 9.0;
        }
        *y += font_size * 1.4;
    }

    fn push_full_wiki_paragraph(
        &mut self,
        text: &str,
        start_x: f32,
        y: &mut f32,
        max_width: f32,
        link_lookup: &HashSet<String>,
        para_idx: usize,
    ) {
        let font_size = 19.0_f32;
        let line_height = 27.5_f32;
        let mut x = start_x;

        for (i, raw) in text.split_whitespace().enumerate() {
            let clean = raw
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                .to_string();
            let lower = clean.to_lowercase();

            // Route-enabled tokens: Every hyperlink, semantic word, and substantive noun can route!
            let (style, link) = if clean.len() >= 4 && link_lookup.contains(&lower) {
                (WordStyle::Hyperlink, Some(clean.clone()))
            } else if clean.len() >= 6 && (i + para_idx * 3) % 7 == 0 {
                (WordStyle::Hyperlink, Some(clean.clone()))
            } else if clean.len() >= 5 && (i + para_idx * 5) % 9 == 0 {
                (WordStyle::SemanticAccent, Some(clean.clone()))
            } else if clean.len() >= 4 {
                (WordStyle::BodyText, Some(clean.clone()))
            } else {
                (WordStyle::BodyText, None)
            };

            let est_w = (raw.len() as f32 * font_size * 0.49) + 4.0;
            if x + est_w > start_x + max_width && x > start_x {
                x = start_x;
                *y += line_height;
            }
            let w = self.push_single_token(raw, Vec2::new(x, *y), font_size, style, link);
            x += w + 8.0;

            // Inject authentic Wikipedia bracketed superscript citations like [15], [71], [103]
            if raw.ends_with('.') && (i + para_idx) % 4 == 0 {
                let cite_num = 11 + ((i * 7 + para_idx * 19) % 94);
                let cite_str = format!("[{}]", cite_num);
                let cw = self.push_single_token(
                    &cite_str,
                    Vec2::new(x - 4.0, *y - 5.0),
                    13.5,
                    WordStyle::Citation,
                    None,
                );
                x += cw + 6.0;
            }
        }
        *y += line_height;
    }

    fn push_single_token(
        &mut self,
        display_text: &str,
        top_left: Vec2,
        font_size: f32,
        style: WordStyle,
        link_target: Option<String>,
    ) -> f32 {
        let width = (display_text.len() as f32 * font_size * 0.49).max(12.0);
        let height = font_size * 1.15;
        let center = Vec2::new(top_left.x + width * 0.5, top_left.y - height * 0.35);
        let clean_word = display_text
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
            .to_string();

        self.words.push(WordToken {
            text: display_text.to_string(),
            clean_word,
            style,
            state: WordState::InPage,
            home_pos: center,
            pos: center,
            vel: Vec2::ZERO,
            angle: 0.0,
            ang_vel: 0.0,
            width,
            height,
            font_size,
            shadow_carve: 0.0,
            mutate_flash: 0.0,
            heat: 0.0,
            box_excite: 0.0,
            box_filled: false,
            tether_anchor: None,
            chip_rgb: (0.45, 0.92, 0.55),
            link_target,
        });
        width
    }

    pub fn poll_bridge(&mut self, lexicon: &mut LexiconEngine) -> bool {
        let mut loaded_new = false;
        while let Ok(res) = self.bridge.res_rx.try_recv() {
            if let Ok(page) = res {
                self.bridge.last_status = format!(
                    "Active Instance: '{}' | Click any highlighted word or link to route next",
                    page.title
                );
                self.load_full_wiki_article(page, lexicon);
                loaded_new = true;
            }
        }
        loaded_new
    }

    pub fn update_words(&mut self, dt: f32, mouse_world: Vec2, mouse_vel: Vec2) {
        let dt_c = dt.min(0.033);
        self.ui_phase += dt_c * 2.2;
        let mouse_speed = mouse_vel.length();

        for p in &mut self.particles {
            p.life += dt_c;
            p.pos += p.vel * dt_c;
            p.vel *= (1.0 - dt_c * 2.5).max(0.0);
        }
        self.particles.retain(|p| p.life < p.max_life);

        for sw in &mut self.shockwaves {
            sw.life += dt_c;
            let prog = (sw.life / sw.max_life).clamp(0.0, 1.0);
            sw.radius = sw.max_radius * (1.0 - (1.0 - prog).powi(3));
        }
        self.shockwaves.retain(|sw| sw.life < sw.max_life);

        for w in &mut self.words {
            w.shadow_carve = (w.shadow_carve - dt_c * 0.48).max(0.0);
            w.mutate_flash = (w.mutate_flash - dt_c * 1.8).max(0.0);
            w.heat = (w.heat - dt_c * 1.2).max(0.0);
            w.box_excite = (w.box_excite - dt_c * 0.65).max(0.0);
            if w.box_excite <= 0.05 {
                w.tether_anchor = None;
            }

            match &mut w.state {
                WordState::InPage => {
                    w.pos = w.pos.lerp(w.home_pos, (dt_c * 9.0).min(1.0));
                    w.angle *= 1.0 - dt_c * 9.0;
                }
                WordState::Tethered { .. } => {}
                WordState::Plucked {
                    timer, max_life, ..
                } => {
                    *timer += dt_c;
                    w.pos += w.vel * dt_c;
                    w.vel *= (1.0 - dt_c * 1.4).max(0.0);
                    w.angle += w.ang_vel * dt_c;
                    w.ang_vel *= (1.0 - dt_c * 1.3).max(0.0);
                    if w.pos.distance(mouse_world) < 48.0 && mouse_speed > 20.0 {
                        w.vel += (w.pos - mouse_world).normalize_or_zero()
                            * (mouse_speed * 0.38).min(220.0);
                        w.ang_vel += 2.2;
                    }
                    if *timer >= *max_life {
                        w.pos = w.pos.lerp(w.home_pos, (dt_c * 4.8).min(1.0));
                        if w.pos.distance(w.home_pos) < 4.0 {
                            w.pos = w.home_pos;
                            w.angle = 0.0;
                            w.state = WordState::InPage;
                        }
                    }
                }
                WordState::Vaporized {
                    timer,
                    respawn_time,
                } => {
                    *timer += dt_c;
                    if *timer >= *respawn_time {
                        w.pos = w.home_pos;
                        w.angle = 0.0;
                        w.heat = 0.0;
                        w.state = WordState::InPage;
                    }
                }
            }
        }
    }

    pub fn check_hyperlink_click(&mut self, click_world: Vec2, open_external_browser: bool) -> bool {
        // 1. Check clickable items inside the Right Infobox & Bottom Cladogram
        for item in &self.infobox.clickables {
            if click_world.x >= item.rect_min.x
                && click_world.x <= item.rect_max.x
                && click_world.y >= item.rect_min.y
                && click_world.y <= item.rect_max.y
            {
                let topic = item.target_topic.clone();
                self.bridge.request_topic(&topic);
                if open_external_browser {
                    open_url_in_browser(&topic);
                }
                return true;
            }
        }

        // 2. Check words in the page (Hyperlinks, Semantic Accents, Boxed/Plucked words, or any substantive word)
        let mut clicked_topic: Option<String> = None;
        for w in &self.words {
            let is_highlighted = w.style == WordStyle::Hyperlink
                || w.style == WordStyle::SemanticAccent
                || w.box_excite > 0.1
                || matches!(w.state, WordState::Plucked { .. } | WordState::Tethered { .. });

            if is_highlighted || w.link_target.is_some() {
                if (click_world.x - w.pos.x).abs() <= w.width * 0.56 + 5.0
                    && (click_world.y - w.pos.y).abs() <= w.height * 0.68 + 5.0
                {
                    if let Some(ref target) = w.link_target {
                        clicked_topic = Some(target.clone());
                        break;
                    } else if w.clean_word.len() >= 3 {
                        clicked_topic = Some(w.clean_word.clone());
                        break;
                    }
                }
            }
        }

        if let Some(topic) = clicked_topic {
            self.bridge.request_topic(&topic);
            if open_external_browser {
                open_url_in_browser(&topic);
            }
            return true;
        }
        false
    }

    pub fn draw(&mut self, lexicon: &LexiconEngine, primary_rgb: (f32, f32, f32)) {
        self.infobox.clickables.clear();

        // 1. Section Divider Lines
        for &(a, b) in &self.divider_lines {
            draw_line(a.x, a.y, b.x, b.y, 1.0, Color::new(0.22, 0.22, 0.26, 0.85));
        }

        // 2. Authentic Wikipedia Right-Hand Taxonomic Infobox (Zero Overflow)
        self.draw_authentic_infobox(primary_rgb);

        // 3. Bottom Phylogenetic Cladogram Widget (matching reference screenshot 002455.jpg)
        self.draw_cladogram_widget(primary_rgb);

        // 4. Live Lexicon Synthesis Box
        let bx = self.synthesis_box.pos.x;
        let by = self.synthesis_box.pos.y;
        let bw = self.synthesis_box.width;
        let bh = self.synthesis_box.height;
        draw_rectangle(bx, by, bw, bh, Color::new(0.015, 0.015, 0.02, 0.96));
        draw_rectangle_lines(bx, by, bw, bh, 1.0, Color::new(0.18, 0.18, 0.22, 0.90));
        let synth_display = truncate_str(&lexicon.live_composition, 78);
        draw_text(
            &synth_display,
            bx + 12.0,
            by + 24.0,
            18.0,
            Color::new(0.92, 0.92, 0.95, 0.96),
        );
        // Glowing cursor block at the end of the synthesis text
        let cursor_x = bx + 16.0 + (synth_display.len() as f32 * 18.0 * 0.48).min(bw - 26.0);
        if (self.ui_phase * 2.0).sin() > -0.2 {
            draw_rectangle(
                cursor_x,
                by + 9.0,
                8.0,
                18.0,
                Color::new(primary_rgb.0, primary_rgb.1, primary_rgb.2, 0.95),
            );
        }

        // 5. Render In-Page Words, Shadow Carve, & In-Page Highlight Boxes
        for w in &self.words {
            match w.state {
                WordState::InPage => {
                    // Deep organic shadow carve under/behind the creature, matching 002439.jpg & 002455.jpg
                    let vis = if w.box_excite > 0.15 {
                        1.0
                    } else {
                        (1.0 - w.shadow_carve * 0.84).clamp(0.15, 1.0)
                    };

                    // Draw neural tether from boxed in-page word to the creature
                    if w.box_excite > 0.12 {
                        let (cr, cg, cb) = w.chip_rgb;
                        let box_alpha = w.box_excite.clamp(0.0, 1.0);
                        if let Some(anchor) = w.tether_anchor {
                            draw_line(
                                w.pos.x,
                                w.pos.y,
                                anchor.x,
                                anchor.y,
                                1.15,
                                Color::new(cr, cg, cb, 0.62 * box_alpha),
                            );
                        }

                        let pad_w = w.width + 8.0;
                        let pad_h = w.height + 5.0;
                        let rx = w.pos.x - pad_w * 0.5;
                        let ry = w.pos.y - pad_h * 0.48;

                        if w.box_filled {
                            // Solid bright block with crisp dark text (like the yellow/green blocks in 002439.jpg)
                            draw_rectangle(
                                rx,
                                ry,
                                pad_w,
                                pad_h,
                                Color::new(cr, cg, cb, 0.94 * box_alpha),
                            );
                            draw_text(
                                &w.text,
                                w.pos.x - w.width * 0.5,
                                w.pos.y + w.height * 0.34,
                                w.font_size,
                                Color::new(0.02, 0.02, 0.03, 0.98),
                            );
                            continue;
                        } else {
                            // Crisp outlined box around the word in-page
                            draw_rectangle(
                                rx,
                                ry,
                                pad_w,
                                pad_h,
                                Color::new(cr * 0.25, cg * 0.25, cb * 0.25, 0.35 * box_alpha),
                            );
                            draw_rectangle_lines(
                                rx,
                                ry,
                                pad_w,
                                pad_h,
                                1.3,
                                Color::new(cr, cg, cb, 0.92 * box_alpha),
                            );
                        }
                    }

                    let mut base_col = match w.style {
                        WordStyle::ArticleTitle | WordStyle::SectionHeader => {
                            Color::new(0.96, 0.96, 0.98, vis.max(0.85))
                        }
                        WordStyle::Hatnote => Color::new(0.78, 0.80, 0.86, vis),
                        WordStyle::ListMarker => Color::new(0.92, 0.92, 0.96, vis),
                        WordStyle::BodyText => {
                            if w.mutate_flash > 0.05 {
                                Color::new(
                                    (0.55 + primary_rgb.0 * 0.45).clamp(0.0, 1.0),
                                    (0.55 + primary_rgb.1 * 0.45).clamp(0.0, 1.0),
                                    (0.55 + primary_rgb.2 * 0.45).clamp(0.0, 1.0),
                                    1.0,
                                )
                            } else {
                                Color::new(0.90, 0.90, 0.92, vis)
                            }
                        }
                        WordStyle::SemanticAccent => {
                            // Warm coral/salmon biological highlight matching 002439.jpg
                            Color::new(0.92, 0.58, 0.54, (vis + 0.15).min(1.0))
                        }
                        WordStyle::Hyperlink => {
                            // Crisp periwinkle/indigo Wikipedia dark-mode link color
                            Color::new(0.52, 0.58, 0.96, (vis + 0.12).min(1.0))
                        }
                        WordStyle::Citation => Color::new(0.50, 0.58, 0.92, vis * 0.92),
                    };

                    if w.heat > 0.02 {
                        base_col.r = (base_col.r + w.heat * 0.85).min(1.0);
                        base_col.g = (base_col.g + w.heat * 0.35).min(1.0);
                        base_col.a = 1.0;
                    }

                    let y_offset = if w.style == WordStyle::Citation {
                        -2.0
                    } else {
                        w.height * 0.36
                    };
                    draw_text(
                        &w.text,
                        w.pos.x - w.width * 0.5,
                        w.pos.y + y_offset,
                        w.font_size,
                        base_col,
                    );
                }
                WordState::Vaporized { timer, .. } => {
                    let glow = (1.0 - (timer * 0.35)).max(0.15);
                    draw_line(
                        w.home_pos.x - w.width * 0.45,
                        w.home_pos.y + 4.0,
                        w.home_pos.x + w.width * 0.45,
                        w.home_pos.y + 4.0,
                        1.6,
                        Color::new(0.95, 0.35, 0.18, glow * 0.65),
                    );
                }
                _ => {
                    // Subtle dark slot left behind when a word is plucked out of the paragraph
                    draw_rectangle(
                        w.home_pos.x - w.width * 0.5,
                        w.home_pos.y - w.height * 0.45,
                        w.width,
                        w.height * 0.9,
                        Color::new(0.08, 0.08, 0.11, 0.35),
                    );
                }
            }
        }

        // 6. Shockwaves & Particles
        for sw in &self.shockwaves {
            let alpha = (1.0 - sw.life / sw.max_life).powi(2) * 0.78;
            let (r, g, b) = sw.color;
            draw_circle_lines(sw.pos.x, sw.pos.y, sw.radius, 1.8, Color::new(r, g, b, alpha));
        }

        for p in &self.particles {
            let alpha = (1.0 - p.life / p.max_life).powf(1.4);
            let (r, g, b) = p.color;
            draw_circle(p.pos.x, p.pos.y, p.size * alpha, Color::new(r, g, b, alpha));
        }

        // 7. Plucked & Tethered Floating Word Chips
        for w in &self.words {
            match w.state {
                WordState::Tethered { .. } => {
                    let (cr, cg, cb) = w.chip_rgb;
                    draw_tilted_word_chip(
                        w.pos,
                        w.width + 14.0,
                        w.height + 10.0,
                        w.angle,
                        &w.text,
                        w.font_size,
                        Color::new(cr * 0.25, cg * 0.25, cb * 0.25, 0.82),
                        Color::new(cr, cg, cb, 0.98),
                        Color::new(1.0, 1.0, 1.0, 1.0),
                        1.7,
                    );
                }
                WordState::Plucked {
                    timer,
                    max_life,
                    filled_highlight,
                } => {
                    let (cr, cg, cb) = w.chip_rgb;
                    let fade = (1.0 - (timer / max_life).powi(3)).clamp(0.25, 1.0);
                    let (fill_col, border_col, text_col) = if filled_highlight {
                        (
                            Color::new(cr, cg, cb, 0.94 * fade),
                            Color::new(1.0, 1.0, 1.0, 0.95 * fade),
                            Color::new(0.02, 0.02, 0.04, 0.98 * fade),
                        )
                    } else {
                        (
                            Color::new(0.02, 0.02, 0.04, 0.92 * fade),
                            Color::new(cr, cg, cb, 0.90 * fade),
                            Color::new(
                                cr * 0.45 + 0.55,
                                cg * 0.45 + 0.55,
                                cb * 0.45 + 0.55,
                                0.98 * fade,
                            ),
                        )
                    };
                    draw_tilted_word_chip(
                        w.pos,
                        w.width + 12.0,
                        w.height + 8.0,
                        w.angle,
                        &w.text,
                        w.font_size,
                        fill_col,
                        border_col,
                        text_col,
                        1.4,
                    );
                }
                _ => {}
            }
        }
    }

    fn draw_authentic_infobox(&mut self, primary_rgb: (f32, f32, f32)) {
        let ibx = self.infobox.pos.x;
        let iby = self.infobox.pos.y;
        let ibw = self.infobox.width;
        let ibh = self.infobox.height;

        // Crisp dark Wikipedia sidebar container with vertical boundary lines (like 002439.jpg)
        draw_rectangle(ibx, iby, ibw, ibh, Color::new(0.01, 0.01, 0.015, 0.96));
        draw_line(ibx, iby, ibx, iby + ibh, 1.2, Color::new(0.22, 0.22, 0.26, 0.90));
        draw_line(
            ibx + ibw,
            iby,
            ibx + ibw,
            iby + ibh,
            1.2,
            Color::new(0.22, 0.22, 0.26, 0.90),
        );
        draw_line(ibx, iby, ibx + ibw, iby, 1.2, Color::new(0.22, 0.22, 0.26, 0.90));
        draw_line(
            ibx,
            iby + ibh,
            ibx + ibw,
            iby + ibh,
            1.2,
            Color::new(0.22, 0.22, 0.26, 0.90),
        );

        // Top Anatomical Wireframe Schematic (matches the spider/arthropod diagram at top of 002439.jpg)
        let diag_cx = ibx + ibw * 0.5;
        let diag_cy = iby + 62.0;
        let wire_col = Color::new(0.55, 0.58, 0.65, 0.78);
        let accent_col = Color::new(primary_rgb.0, primary_rgb.1, primary_rgb.2, 0.85);

        // Central cephalothorax & abdomen chambers
        draw_rectangle_lines(diag_cx - 18.0, diag_cy - 28.0, 36.0, 26.0, 1.2, wire_col);
        draw_rectangle_lines(diag_cx - 12.0, diag_cy - 22.0, 24.0, 14.0, 1.0, accent_col);
        draw_rectangle_lines(diag_cx - 22.0, diag_cy + 2.0, 44.0, 38.0, 1.4, wire_col);
        draw_rectangle_lines(diag_cx - 14.0, diag_cy + 10.0, 28.0, 22.0, 1.1, wire_col);

        // Articulated schematic limbs
        for i in 0..4 {
            let y_off = -18.0 + i as f32 * 11.0;
            let spread = 16.0 + (i as f32 - 1.5).abs() * 8.0;
            let wave = (self.ui_phase + i as f32 * 0.8).sin() * 3.0;
            for &side in &[-1.0_f32, 1.0_f32] {
                let hip = Vec2::new(diag_cx + side * 18.0, diag_cy + y_off);
                let knee = Vec2::new(diag_cx + side * (44.0 + spread), diag_cy + y_off - 12.0 + wave);
                let tip = Vec2::new(diag_cx + side * (74.0 + spread), diag_cy + y_off + 10.0 - wave);
                draw_line(hip.x, hip.y, knee.x, knee.y, 1.1, wire_col);
                draw_line(knee.x, knee.y, tip.x, tip.y, 1.0, wire_col);
                draw_circle(knee.x, knee.y, 2.0, accent_col);
            }
        }

        let mut cur_y = iby + 142.0;

        // Centered Taxon / Article Title (Truncated safely so it never overflows ibw)
        let safe_title = truncate_str(&self.infobox.title, 26);
        let title_w = safe_title.len() as f32 * 16.5 * 0.48;
        draw_text(
            &safe_title,
            ibx + ((ibw - title_w) * 0.5).max(14.0),
            cur_y,
            16.5,
            Color::new(0.84, 0.85, 0.90, 0.95),
        );
        cur_y += 28.0;

        // "Scientific classification" Header in warm coral/rose (exact match to 002439.jpg)
        let sci_hdr = "Scientific classification";
        let sci_w = sci_hdr.len() as f32 * 17.5 * 0.48;
        draw_text(
            sci_hdr,
            ibx + (ibw - sci_w) * 0.5,
            cur_y,
            17.5,
            Color::new(0.88, 0.56, 0.54, 0.98),
        );
        cur_y += 28.0;

        // Two-Column Aligned Classification Rows (Zero \n, Zero Overflow)
        let col_left = ibx + 18.0;
        let col_right = ibx + 152.0;
        for (label, val) in &self.infobox.taxonomy_rows {
            draw_text(
                label,
                col_left,
                cur_y,
                16.0,
                Color::new(0.88, 0.89, 0.93, 0.95),
            );
            let val_col = if label == "Order:" || label == "Phylum:" {
                Color::new(0.88, 0.56, 0.54, 0.96)
            } else {
                Color::new(0.92, 0.93, 0.97, 0.96)
            };
            draw_text(val, col_right, cur_y, 16.0, val_col);

            // Register each classification value as a clickable route target
            self.infobox.clickables.push(InfoboxClickable {
                rect_min: Vec2::new(col_right - 4.0, cur_y - 15.0),
                rect_max: Vec2::new(ibx + ibw - 10.0, cur_y + 5.0),
                target_topic: val.clone(),
            });

            cur_y += 26.0;
        }

        cur_y += 12.0;
        // Centered Sub-group Header
        let sub_hdr = truncate_str(&self.current_title, 22);
        let sub_w = sub_hdr.len() as f32 * 18.0 * 0.50;
        draw_text(
            &sub_hdr,
            ibx + ((ibw - sub_w) * 0.5).max(16.0),
            cur_y,
            18.0,
            Color::new(0.95, 0.96, 0.99, 0.98),
        );
        cur_y += 26.0;

        // Bulleted Sub-taxa List (indented hierarchy matching 002439.jpg)
        for (idx, taxon) in self.infobox.sub_taxa.iter().enumerate() {
            let indent = if idx >= 2 { 44.0 } else { 24.0 };
            let line_str = format!("• {}", truncate_str(taxon, 20));
            draw_text(
                &line_str,
                ibx + indent,
                cur_y,
                16.0,
                Color::new(0.86, 0.56, 0.54, 0.95),
            );

            self.infobox.clickables.push(InfoboxClickable {
                rect_min: Vec2::new(ibx + indent, cur_y - 15.0),
                rect_max: Vec2::new(ibx + ibw - 14.0, cur_y + 5.0),
                target_topic: taxon.clone(),
            });
            cur_y += 24.0;
        }

        cur_y += 10.0;
        let see_str = format!("See {} taxonomy.", truncate_str(&self.current_title, 15));
        draw_text(
            &see_str,
            ibx + 22.0,
            cur_y,
            16.0,
            Color::new(0.88, 0.89, 0.94, 0.92),
        );
        cur_y += 32.0;

        // Diversity Footer
        let div_hdr = "Diversity[1]";
        let div_w = div_hdr.len() as f32 * 17.5 * 0.48;
        draw_text(
            div_hdr,
            ibx + (ibw - div_w) * 0.5,
            cur_y,
            17.5,
            Color::new(0.88, 0.56, 0.54, 0.96),
        );
        cur_y += 26.0;

        let stat_w = self.infobox.diversity_stat.len() as f32 * 15.5 * 0.48;
        draw_text(
            &self.infobox.diversity_stat,
            ibx + ((ibw - stat_w) * 0.5).max(14.0),
            cur_y,
            15.5,
            Color::new(0.86, 0.56, 0.54, 0.92),
        );
    }

    fn draw_cladogram_widget(&mut self, primary_rgb: (f32, f32, f32)) {
        let cx = self.cladogram_pos.x;
        let cy = self.cladogram_pos.y;
        let w = 620.0;
        let h = 76.0;

        draw_rectangle(cx, cy, w, h, Color::new(0.01, 0.01, 0.015, 0.95));
        draw_rectangle_lines(cx, cy, w, h, 1.0, Color::new(0.16, 0.16, 0.20, 0.80));

        // Phylogenetic tree branch lines
        let tree_col = Color::new(0.45, 0.48, 0.58, 0.75);
        draw_line(cx + 24.0, cy + 18.0, cx + 24.0, cy + 52.0, 1.2, tree_col);
        draw_line(cx + 24.0, cy + 18.0, cx + 180.0, cy + 18.0, 1.2, tree_col);
        draw_line(cx + 180.0, cy + 18.0, cx + 180.0, cy + 52.0, 1.2, tree_col);

        let t0 = self
            .cladogram_taxa
            .first()
            .cloned()
            .unwrap_or_else(|| "Notostigmophora".to_string());
        let t1 = self
            .cladogram_taxa
            .get(1)
            .cloned()
            .unwrap_or_else(|| "Scutigeromorpha".to_string());

        draw_text(
            &t0,
            cx + 18.0,
            cy + 62.0,
            17.5,
            Color::new(0.52, 0.58, 0.96, 0.96),
        );
        self.infobox.clickables.push(InfoboxClickable {
            rect_min: Vec2::new(cx + 16.0, cy + 44.0),
            rect_max: Vec2::new(cx + 165.0, cy + 68.0),
            target_topic: t0,
        });

        draw_text(
            &t1,
            cx + 175.0,
            cy + 62.0,
            17.5,
            Color::new(0.52, 0.58, 0.96, 0.96),
        );
        self.infobox.clickables.push(InfoboxClickable {
            rect_min: Vec2::new(cx + 172.0, cy + 44.0),
            rect_max: Vec2::new(cx + 330.0, cy + 68.0),
            target_topic: t1,
        });

        // Animated undulating centipede/waveform icon beside the taxon (matching bottom of 002455.jpg)
        let wave_start_x = cx + 338.0;
        let wave_y = cy + 57.0;
        for s in 0..14 {
            let sx = wave_start_x + s as f32 * 4.2;
            let sy = wave_y + (self.ui_phase * 3.0 + s as f32 * 0.55).sin() * 3.2;
            draw_circle(
                sx,
                sy,
                1.4,
                Color::new(0.85, 0.88, 0.95, 0.90),
            );
            draw_line(
                sx,
                sy - 3.5,
                sx - 1.5,
                sy + 3.5,
                0.8,
                Color::new(primary_rgb.0, primary_rgb.1, primary_rgb.2, 0.65),
            );
        }
    }
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn draw_tilted_word_chip(
    center: Vec2,
    width: f32,
    height: f32,
    angle: f32,
    text: &str,
    font_size: f32,
    fill: Color,
    border: Color,
    text_color: Color,
    border_thickness: f32,
) {
    let c = angle.cos();
    let s = angle.sin();
    let right = Vec2::new(c, s) * (width * 0.5);
    let down = Vec2::new(-s, c) * (height * 0.5);
    let p0 = center - right - down;
    let p1 = center + right - down;
    let p2 = center + right + down;
    let p3 = center - right + down;
    draw_triangle(p0, p1, p2, fill);
    draw_triangle(p0, p2, p3, fill);
    draw_line(p0.x, p0.y, p1.x, p1.y, border_thickness, border);
    draw_line(p1.x, p1.y, p2.x, p2.y, border_thickness, border);
    draw_line(p2.x, p2.y, p3.x, p3.y, border_thickness, border);
    draw_line(p3.x, p3.y, p0.x, p0.y, border_thickness, border);
    draw_text(
        text,
        center.x - width * 0.42,
        center.y + font_size * 0.32,
        font_size,
        text_color,
    );
}