#![windows_subsystem = "windows"]
/// Mini Screen saver, scritto in RUST - Giorgio Ruffa (c)2026
/// Funziona su Linux, macOS e Windows
// 	PREMERE ESC PER USCIRE 
// Importa dalla libreria egui gli strumenti per finestra, disegno, tastiera e coordinate.
use eframe::egui::{self, Color32, Key, Pos2, Shape, Stroke, Ui, Vec2, ViewportCommand};
// `Instant` permette di misurare il tempo trascorso, utile per l'animazione.
use std::time::Instant;

// `derive` genera automaticamente Clone e Copy: questo tipo semplice può essere copiato.
// Vec3 rappresenta un vettore o punto nello spazio tridimensionale (x, y, z).
#[derive(Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

// `impl` definisce i metodi disponibili per il tipo Vec3. `Self` significa Vec3.
impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    // Sottrae due vettori componente per componente. `&` indica un riferimento, senza trasferire il valore.
    fn sub(&self, other: &Vec3) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    // Prodotto scalare: misura quanto due direzioni sono allineate.
    fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    // Prodotto vettoriale: restituisce un vettore perpendicolare ai due vettori di partenza.
    fn cross(&self, other: &Vec3) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    // Calcola la lunghezza del vettore usando il teorema di Pitagora in 3D.
    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    // Normalizzare mantiene la direzione ma porta la lunghezza a 1; il controllo evita divisioni per zero.
    fn normalized(&self) -> Self {
        let len = self.length();
        if len > 0.0_f32 {
            Self::new(self.x / len, self.y / len, self.z / len)
        } else {
            *self
        }
    }

    // Applica rotazioni attorno agli assi X, Y e Z. Gli angoli sono espressi in radianti.
    fn rotate(&self, rx: f32, ry: f32, rz: f32) -> Self {
        let y1 = self.y * rx.cos() - self.z * rx.sin();
        let z1 = self.y * rx.sin() + self.z * rx.cos();
        let x1 = self.x;

        let x2 = x1 * ry.cos() + z1 * ry.sin();
        let z2 = -x1 * ry.sin() + z1 * ry.cos();
        let y2 = y1;

        let x3 = x2 * rz.cos() - y2 * rz.sin();
        let y3 = x2 * rz.sin() + y2 * rz.cos();
        let z3 = z2;

        Self::new(x3, y3, z3)
    }
}

// Converte coordinate HSV in RGB modulando il valore (Value) con l'intensità luminosa
// Converte un colore dal modello HSV (tonalità, saturazione, valore) al formato RGB usato da egui.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color32 {
    let h_norm = (h.fract() + 1.0_f32).fract() * 6.0_f32;
    let i = h_norm.floor() as i32;
    let f = h_norm - i as f32;
    let p = v * (1.0_f32 - s);
    let q = v * (1.0_f32 - s * f);
    let t = v * (1.0_f32 - s * (1.0_f32 - f));

    // `match` sceglie una combinazione RGB in base al settore della ruota dei colori.
    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    Color32::from_rgb(
        (r.clamp(0.0_f32, 1.0_f32) * 255.0_f32) as u8,
        (g.clamp(0.0_f32, 1.0_f32) * 255.0_f32) as u8,
        (b.clamp(0.0_f32, 1.0_f32) * 255.0_f32) as u8,
    )
}

// Stato dell'app: posizione e velocità, rotazione e istanti temporali dell'animazione.
struct ScreensaverApp {
    pos: Vec2,
    vel: Vec2,
    rot: Vec3,
    rot_speed: Vec3,
    start_time: Instant,
    last_frame: Instant,
}

// `Default` definisce i valori iniziali usati quando viene creata l'applicazione.
impl Default for ScreensaverApp {
    fn default() -> Self {
        Self {
            pos: Vec2::new(300.0_f32, 300.0_f32),
            vel: Vec2::new(190.0_f32, 150.0_f32),
            rot: Vec3::new(0.0_f32, 0.0_f32, 0.0_f32),
            rot_speed: Vec3::new(0.9_f32, 1.3_f32, 0.6_f32),
            start_time: Instant::now(),
            last_frame: Instant::now(),
        }
    }
}

// Implementa il trait `eframe::App`: così eframe sa come aggiornare e disegnare questa app.
impl eframe::App for ScreensaverApp {
    // Questa funzione aggiorna l'interfaccia. `&mut self` consente di modificare lo stato dell'app.
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // `ctx` è il contesto egui, usato per leggere l'input e comandare la finestra.
        let ctx = ui.ctx().clone();

        // Uscita con ESC
        // La closure `|i| ...` controlla se il tasto ESC è stato premuto.
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        // Modifica dinamica velocità (Frecce Su / Giù)
        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
            self.vel *= 1.2_f32;
            self.rot_speed.x *= 1.2_f32;
            self.rot_speed.y *= 1.2_f32;
            self.rot_speed.z *= 1.2_f32;
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
            self.vel *= 0.8_f32;
            self.rot_speed.x *= 0.8_f32;
            self.rot_speed.y *= 0.8_f32;
            self.rot_speed.z *= 0.8_f32;
        }

        // `dt` è il tempo trascorso dall'ultimo fotogramma. Il limite evita salti eccessivi dopo una pausa.
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().min(0.05_f32);
        self.last_frame = now;
        let total_time = (now - self.start_time).as_secs_f32();

        // Crea il pannello centrale della finestra e lo riempie di nero.
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::BLACK))
            .show(ui, |ui| {
                let rect = ui.max_rect();
                let screen_w = rect.width();
                let screen_h = rect.height();

                if screen_w <= 0.0_f32 || screen_h <= 0.0_f32 {
                    return;
                }

                // Lato del cubo pari a 1/5 dell'altezza schermo
                // La dimensione del cubo dipende dall'altezza della finestra.
                let cube_side = screen_h / 5.0_f32;
                let half_side = cube_side / 2.0_f32;
                let bounding_radius = half_side * 3.0_f32.sqrt();

                // Rimbalzo sui bordi
                // Aggiorna la posizione con velocità × tempo: l'animazione resta regolare anche se cambia il frame rate.
                self.pos += self.vel * dt;

                if self.pos.x - bounding_radius < 0.0_f32 {
                    self.pos.x = bounding_radius;
                    self.vel.x = self.vel.x.abs();
                } else if self.pos.x + bounding_radius > screen_w {
                    self.pos.x = screen_w - bounding_radius;
                    self.vel.x = -self.vel.x.abs();
                }

                if self.pos.y - bounding_radius < 0.0_f32 {
                    self.pos.y = bounding_radius;
                    self.vel.y = self.vel.y.abs();
                } else if self.pos.y + bounding_radius > screen_h {
                    self.pos.y = screen_h - bounding_radius;
                    self.vel.y = -self.vel.y.abs();
                }

                // Rotazione angolare
                self.rot.x += self.rot_speed.x * dt;
                self.rot.y += self.rot_speed.y * dt;
                self.rot.z += self.rot_speed.z * dt;

                // Gli otto vertici del cubo, espressi rispetto al centro del cubo (coordinate locali).
                let local_vertices = [
                    Vec3::new(-half_side, -half_side, -half_side),
                    Vec3::new( half_side, -half_side, -half_side),
                    Vec3::new( half_side,  half_side, -half_side),
                    Vec3::new(-half_side,  half_side, -half_side),
                    Vec3::new(-half_side, -half_side,  half_side),
                    Vec3::new( half_side, -half_side,  half_side),
                    Vec3::new( half_side,  half_side,  half_side),
                    Vec3::new(-half_side,  half_side,  half_side),
                ];

                // Sei facce, ognuna definita dagli indici dei suoi quattro vertici. `usize` è il tipo usato per gli indici.
                let faces: [[usize; 4]; 6] = [
                    [4, 5, 6, 7], // Anteriore (+Z)
                    [1, 0, 3, 2], // Posteriore (-Z)
                    [0, 1, 5, 4], // Superiore (-Y)
                    [3, 7, 6, 2], // Inferiore (+Y)
                    [1, 2, 6, 5], // Destra (+X)
                    [0, 4, 7, 3], // Sinistra (-X)
                ];

                // La distanza focale regola l'effetto prospettico; `max` impone un minimo di 600.
                let focal_distance = screen_h.max(600.0_f32);
                let camera_pos = Vec3::new(screen_w / 2.0_f32, screen_h / 2.0_f32, -focal_distance);

                // Sorgente luminosa direzionale posta in alto a sinistra e davanti allo schermo (verso l'osservatore)
                let light_dir = Vec3::new(-0.45_f32, -0.65_f32, -0.60_f32).normalized();

                // Trasformazione vertici nello spazio mondo 3D
                // `.iter()` visita i vertici, `.map()` trasforma ciascuno e `.collect()` raccoglie i risultati in un Vec.
                let world_vertices: Vec<Vec3> = local_vertices
                    .iter()
                    .map(|v| {
                        let rotated = v.rotate(self.rot.x, self.rot.y, self.rot.z);
                        Vec3::new(rotated.x + self.pos.x, rotated.y + self.pos.y, rotated.z)
                    })
                    .collect();

                // Proiezione prospettica 2D
                // Trasforma i punti 3D in coordinate 2D: la prospettiva fa apparire più piccoli i punti più lontani.
                let projected_points: Vec<Pos2> = world_vertices
                    .iter()
                    .map(|v| {
                        let dx = v.x - camera_pos.x;
                        let dy = v.y - camera_pos.y;
                        let dz = v.z - camera_pos.z;

                        let factor = focal_distance / dz;
                        Pos2::new(camera_pos.x + dx * factor, camera_pos.y + dy * factor)
                    })
                    .collect();

                let color_shift = total_time / 10.0_f32;

                // Dati necessari per disegnare una faccia: vertici proiettati, colore e profondità.
                struct RenderFace {
                    pts: [Pos2; 4],
                    color: Color32,
                    depth: f32,
                }

                // Crea un vettore vuoto in cui inserire le facce visibili. `mut` permette di modificarlo.
                let mut render_queue = Vec::new();

                for (i, face_indices) in faces.iter().enumerate() {
                    let v0 = world_vertices[face_indices[0]];
                    let v1 = world_vertices[face_indices[1]];
                    let v2 = world_vertices[face_indices[2]];
                    let v3 = world_vertices[face_indices[3]];

                    // Due lati della faccia: il loro prodotto vettoriale fornisce la normale alla faccia.
                    let edge1 = v1.sub(&v0);
                    let edge2 = v2.sub(&v0);
                    let normal = edge1.cross(&edge2);

                    let center = Vec3::new(
                        (v0.x + v1.x + v2.x + v3.x) * 0.25_f32,
                        (v0.y + v1.y + v2.y + v3.y) * 0.25_f32,
                        (v0.z + v1.z + v2.z + v3.z) * 0.25_f32,
                    );
                    let view_dir = center.sub(&camera_pos);

                    let norm_len = normal.length();
                    let view_len = view_dir.length();

                    if norm_len > 0.0_f32 && view_len > 0.0_f32 {
                        // Il coseno dell'angolo tra normale e direzione della camera aiuta a capire se la faccia è visibile.
                        let cos_theta = normal.dot(&view_dir) / (norm_len * view_len);

                        // Backface culling e filtro viste radenti (< ~2°)
                        if cos_theta < -0.035_f32 {
                            // Normale unitaria uscente
                            let unit_normal = Vec3::new(
                                normal.x / norm_len,
                                normal.y / norm_len,
                                normal.z / norm_len,
                            );

                            // Modello di illuminazione di Lambert:
                            // Ambient (25%): impedisce alle facce in ombra di diventare totalmente nere
                            // Diffuse (75%): proporzionale al coseno tra la normale e il raggio di luce
                            // Una faccia riceve luce se è rivolta verso la sorgente; `max(0)` scarta valori negativi.
                            let n_dot_l = unit_normal.dot(&light_dir).max(0.0_f32);
                            let ambient = 0.25_f32;
                            let diffuse = 0.75_f32 * n_dot_l;
                            let intensity = (ambient + diffuse).clamp(0.0_f32, 1.0_f32);

                            let face_base_hue = (i as f32 / 6.0_f32) + color_shift;
                            // Modulazione del parametro Value in HSV per variare l'intensità luminosa
                            let face_color = hsv_to_rgb(face_base_hue, 0.85_f32, 0.95_f32 * intensity);

                            let pts = [
                                projected_points[face_indices[0]],
                                projected_points[face_indices[1]],
                                projected_points[face_indices[2]],
                                projected_points[face_indices[3]],
                            ];

                            render_queue.push(RenderFace {
                                pts,
                                color: face_color,
                                depth: center.z,
                            });
                        }
                    }
                }

                // Painter's algorithm: disegno dalla faccia più lontana alla più vicina
                // Ordina le facce dalla più lontana alla più vicina. `partial_cmp` confronta le profondità numeriche.
                render_queue.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap());

                let painter = ui.painter();
                let edge_stroke = Stroke::new(1.5_f32, Color32::from_black_alpha(200));

                // Disegna le facce una alla volta, nell'ordine appena stabilito.
                for face in render_queue {
                    // Riempimento convesso privo di contorno (evita esplosione miter join)
                    painter.add(Shape::convex_polygon(
                        face.pts.to_vec(),
                        face.color,
                        Stroke::NONE,
                    ));

                    // Tracciamento dei bordi con segmenti lineari isolati
                    for j in 0..4 {
                        let p_start = face.pts[j];
                        let p_end = face.pts[(j + 1) % 4];
                        painter.line_segment([p_start, p_end], edge_stroke);
                    }
                }
            });

        // Richiede un nuovo fotogramma, così l'animazione continua.
        ctx.request_repaint();
    }
}

// Punto d'ingresso del programma. `Result` permette di segnalare eventuali errori di avvio.
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_active(true),
        ..Default::default()
    };

    // Avvia la finestra nativa; la closure crea l'app con i valori iniziali di Default.
    eframe::run_native(
        "Screensaver Cubo 3D",
        options,
        Box::new(|_cc| Ok(Box::new(ScreensaverApp::default()))),
    )
}
