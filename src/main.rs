use core::f32;

// Importerer nødvendige biblioteker for spillmotoren
use ggez;
use ggez::{Context, GameResult};
use ggez::graphics;
use ggez::nalgebra as na;  // Matematikkbibliotek for vektorer og punkter
use ggez::event;
use ggez::input::keyboard::{self, KeyCode}; //Leser koden til taster som trykkes på.
use rand::{self, thread_rng, Rng};

// Konstanter for racket-dimensjoner
// Konstanter for racket-dimensjoner
const RACKET_HEIGHT: f32 = 100.0;           // Høyde på racket i piksler
const RACKET_WIDTH: f32 = 20.0;             // Bredde på racket i piksler
const RACKET_WIDTH_HALF: f32 = RACKET_WIDTH*0.5;   // Halv bredde for sentreringsberegninger
const RACKET_HEIGHT_HALF:f32 = RACKET_HEIGHT*0.5;  // Halv høyde for sentreringsberegninger
const PLAYER_SPEED:f32 = 600.0;             // Hastighet på spillere (piksler per sekund)

// Konstanter for spillbane-layout
const PADDING: f32 = 40.0;                  // Avstand fra kant til racket
const MIDDLE_LINE_W: f32 = 2.0;             // Bredde på midtlinjen

// Konstanter for ball-dimensjoner
const BALL_SIZE: f32 = 30.0;                // Størrelse på ballen (kvadratisk)
const BALL_SIZE_HALF: f32 = BALL_SIZE * 0.5; // Halv ballstørrelse for kollisjonberegninger

// Ballhastighet
const BALL_SPEED: f32 = 200.0;              // Grunnhastighet for ballen (piksler per sekund)

// Funksjon som hindrer racketene å gå utenfor skjermen i vertikal akse.
// Begrenser en verdi mellom minimum og maksimum grenser
fn clamp(value: &mut f32, low:f32, high: f32){
    if *value < low{
        *value = low;        // Setter til minimum hvis under grensen
    }else if *value > high{
        *value = high;       // Setter til maksimum hvis over grensen
    }
}

// Funksjon for å flytte racketer basert på tastaturinnput
fn move_racket (pos: &mut na::Point2<f32>, keykode:KeyCode, y_dir: f32, ctx: &mut Context){
    let dt = ggez::timer::delta(ctx).as_secs_f32();  // Tid siden forrige frame
    let screen_h = graphics::drawable_size(ctx).1;   // Skjermhøyde

    // Sjekker om den spesifiserte tasten er trykket ned
    if keyboard::is_key_pressed(ctx, keykode){
        pos.y += y_dir * PLAYER_SPEED * dt;          // Flytter racket i angitt retning
    }
    // Hindrer racket i å gå utenfor skjermen (topp/bunn)
    clamp(&mut pos.y,RACKET_HEIGHT_HALF, screen_h-RACKET_HEIGHT_HALF);
}

// Funksjon som setter tilfeldig retning på en vektor
// Brukes for å gi ballen en tilfeldig startretning
fn randomize_vec(vec: &mut na::Vector2<f32>, x: f32, y: f32){
    let mut rng = thread_rng();                      // Oppretter tilfeldig tallgenerator
    // Setter x-retning tilfeldig (positiv eller negativ)
    vec.x = match rng.gen_bool(0.5){
        true=>x,         // 50% sjanse for positiv retning
        false => -x,     // 50% sjanse for negativ retning
    };
    // Setter y-retning tilfeldig (positiv eller negativ)
    vec.y = match rng.gen_bool(0.5){
        true => y,       // 50% sjanse for positiv retning (nedover)
        false => -y,     // 50% sjanse for negativ retning (oppover)
    };
}


// Hovedstrukturen som holder spillets tilstand
struct MainState{
    player_1_pos: na::Point2<f32>,  // Posisjon til spiller 1 (venstre racket)
    player_2_pos: na::Point2<f32>,  // Posisjon til spiller 2 (høyre racket)
    ball_pos: na::Point2<f32>,      // Posisjon til ballen
    ball_vel: na::Vector2<f32>,     // Fart til ballen.
    player_1_score: i32,            // Poeng til spiller 1.
    player_2_score:i32,             // Poeng til spiller 2.

}

impl MainState{
    // Oppretter en ny spilltilstand med startposisjoner
    pub fn new(ctx: &mut Context)->Self{
        // Henter skjermstørrelse
        let (screen_w, screen_h)= graphics::drawable_size(ctx);
        let (screen_w_half, screen_h_half) = (screen_w *0.5, screen_h * 0.5);

        let mut ball_vel = na::Vector2::new(0.0,0.0);
        randomize_vec(&mut ball_vel, BALL_SPEED,  BALL_SPEED);

        
        MainState{
            // Plasserer spiller 1 på venstre side, vertikalt sentrert
            player_1_pos: na::Point2::new(RACKET_WIDTH_HALF + PADDING, screen_h_half),
            // Plasserer spiller 2 på høyre side, vertikalt sentrert
            player_2_pos: na::Point2::new(screen_w-RACKET_WIDTH_HALF - PADDING, screen_h_half),
            // Plasserer ballen i midten av skjermen
            ball_pos: na::Point2::new(screen_w_half, screen_h_half),
            ball_vel,
            player_1_score: 0,
            player_2_score: 0,
        }   
    }
}

// Implementerer EventHandler-traitet for spillogikk
impl event::EventHandler for MainState{
    // Oppdaterer spilltilstanden hvert frame
    fn update(&mut self, ctx: &mut Context)->GameResult{
        let dt = ggez::timer::delta(ctx).as_secs_f32();      // Tid siden forrige frame
        let (screen_w, screen_h) = graphics::drawable_size(ctx); // Skjermens dimensjoner
        
        // Håndterer spillerinput for racket-bevegelse
        move_racket(&mut self.player_1_pos, KeyCode::W, -1.0, ctx);    // Spiller 1 opp (W)
        move_racket(&mut self.player_1_pos, KeyCode::S, 1.0, ctx);     // Spiller 1 ned (S)
        move_racket(&mut self.player_2_pos, KeyCode::Up, -1.0, ctx);   // Spiller 2 opp (Pil opp)
        move_racket(&mut self.player_2_pos, KeyCode::Down, 1.0, ctx);  // Spiller 2 ned (Pil ned)

        // Oppdaterer ballens posisjon basert på hastighet og tid
        self.ball_pos += self.ball_vel * dt;

        // Sjekker om ballen har gått ut på venstre side (Spiller 2 scorer)
        if self.ball_pos.x < 0.0{
            self.ball_pos.x = screen_w * 0.5;                          // Resetter ball til midten
            self.ball_pos.y = screen_h * 0.5;
            randomize_vec(&mut self.ball_vel, BALL_SPEED, BALL_SPEED);  // Gir tilfeldig retning
            self.player_2_score +=1;                                   // Øker spiller 2s poeng
        }

        // Sjekker om ballen har gått ut på høyre side (Spiller 1 scorer)
        if self.ball_pos.x > screen_w{
            self.ball_pos.x = screen_w * 0.5;                          // Resetter ball til midten
            self.ball_pos.y = screen_h * 0.5;
            randomize_vec(&mut self.ball_vel, BALL_SPEED, BALL_SPEED);  // Gir tilfeldig retning
            self.player_1_score +=1;                                   // Øker spiller 1s poeng
        }

        // Håndterer kollisjon med topp og bunn av skjermen
        if self.ball_pos.y < BALL_SIZE_HALF{
            self.ball_pos.y = BALL_SIZE_HALF;                          // Holder ballen inne på skjermen
            self.ball_vel.y = self.ball_vel.y.abs();                   // Reverserer bare y-hastighet (spretter nedover)
        }else if self.ball_pos.y > screen_h-BALL_SIZE_HALF{
            self.ball_pos.y = screen_h-BALL_SIZE_HALF;                 // Holder ballen inne på skjermen
            self.ball_vel.y = -self.ball_vel.y.abs();                  // Reverserer bare y-hastighet (spretter oppover)
        }

        // Kollisjondeteksjon for spiller 1 (venstre racket)
        // Sjekker om ballen overlapper med racket i både x- og y-retning
        let intersect_player_1 =
            self.ball_pos.x - BALL_SIZE_HALF < self.player_1_pos.x + RACKET_WIDTH_HALF    // Ball venstre kant < racket høyre kant
            && self.ball_pos.x + BALL_SIZE_HALF > self.player_1_pos.x - RACKET_WIDTH_HALF // Ball høyre kant > racket venstre kant
            && self.ball_pos.y - BALL_SIZE_HALF < self.player_1_pos.y +RACKET_HEIGHT_HALF // Ball topp < racket bunn
            && self.ball_pos.y + BALL_SIZE_HALF > self.player_1_pos.y - RACKET_HEIGHT_HALF;// Ball bunn > racket topp

        if intersect_player_1{
            self.ball_vel.x = self.ball_vel.x.abs();                   // Sender ballen til høyre (positiv x-retning)
        }

        // Kollisjondeteksjon for spiller 2 (høyre racket)
        // Samme logikk som for spiller 1
        let intersect_player_2 =
            self.ball_pos.x - BALL_SIZE_HALF < self.player_2_pos.x + RACKET_WIDTH_HALF
            && self.ball_pos.x + BALL_SIZE_HALF > self.player_2_pos.x - RACKET_WIDTH_HALF
            && self.ball_pos.y - BALL_SIZE_HALF < self.player_2_pos.y +RACKET_HEIGHT_HALF
            && self.ball_pos.y + BALL_SIZE_HALF > self.player_2_pos.y - RACKET_HEIGHT_HALF;

        if intersect_player_2{
            self.ball_vel.x = -self.ball_vel.x.abs();                  // Sender ballen til venstre (negativ x-retning)
        }
        
        
        Ok(())
    }
    
    // Tegner alle spillobjekter på skjermen
    fn draw(&mut self, ctx: &mut Context)->GameResult{
        // Tømmer skjermen med svart bakgrunn
        graphics::clear(ctx, graphics::BLACK);

        // === OPPRETTER GEOMETRI FOR ALLE SPILLOBJEKTER ===
        
        // Oppretter geometri for rackets (sentrert rundt origo for enkel posisjonering)
        let racket_rect = graphics::Rect::new(-RACKET_WIDTH_HALF, -RACKET_HEIGHT_HALF, RACKET_WIDTH, RACKET_HEIGHT);
        let racket_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), racket_rect, graphics::WHITE)?;

        // Oppretter geometri for ballen (sentrert rundt origo)
        let ball_rect = graphics::Rect::new(-BALL_SIZE_HALF, -BALL_SIZE_HALF, BALL_SIZE,BALL_SIZE);
        let ball_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), ball_rect, graphics::WHITE)?;

        // Oppretter geometri for midtlinjen (vertikal linje gjennom midten av skjermen)
        let screen_h = graphics::drawable_size(ctx).1;
        let middle_rect = graphics::Rect::new(-MIDDLE_LINE_W *0.5, 0.0, MIDDLE_LINE_W, screen_h);
        let middle_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), middle_rect, graphics::WHITE)?;

        // === TEGNER ALLE OBJEKTER ===
        
        let mut draw_param = graphics::DrawParam::default();

        // Tegner midtlinjen
        let screen_middle_x = graphics::drawable_size(ctx).0 * 0.5;    // Finner midtpunkt av skjermen
        draw_param.dest = [screen_middle_x, 0.0].into();               // Setter posisjon for midtlinje
        graphics::draw(ctx, &middle_mesh, draw_param)?;

        // Tegner spiller 1 sin racket (venstre side)
        draw_param.dest = self.player_1_pos.into();
        graphics::draw(ctx, &racket_mesh, draw_param)?;

        // Tegner spiller 2 sin racket (høyre side)
        draw_param.dest = self.player_2_pos.into();
        graphics::draw(ctx, &racket_mesh, draw_param)?;

        // Tegner ballen
        draw_param.dest = self.ball_pos.into();
        graphics::draw(ctx, &ball_mesh, draw_param)?;

        // === TEGNER POENGSUM ===
        
        // Oppretter scoretekst med mellomrom for å separere poengene visuelt
        let score_text = graphics::Text::new(format!("{}           {}", self.player_1_score, self.player_2_score));
        let screen_w= graphics::drawable_size(ctx).0;
        let screen_w_half = screen_w * 0.5;
        let mut score_pos = na::Point2::new(screen_w_half, 50.0);      // Plasserer score øverst på skjermen
        let (score_text_w, score_text_h) = score_text.dimensions(ctx); // Henter tekstdimensjoner
        // Sentrerer teksten horisontalt og justerer vertikalt
        score_pos -= na::Vector2::new(score_text_w as f32 *0.5, score_text_h as f32 * 0.5);
        draw_param.dest = score_pos.into();
        graphics::draw(ctx, &score_text, draw_param)?;

        // Viser alt som er tegnet på skjermen (buffer swap)
        graphics::present(ctx)?;
        Ok(())
    }
}

// Hovedfunksjonen som starter spillet
fn main() -> GameResult{
    // Oppretter en spillkontekst med tittel og utviklernavn
    let cb = ggez::ContextBuilder::new("Pong_0","TerjeIdland");
    let (ctx, event_loop) = &mut cb.build()?;
    
    // Setter vindustittel som vises i tittelbaren
    graphics::set_window_title(ctx,"PONG");
    
    // Oppretter spilltilstand og starter hovedløkken
    // Dette starter den evige spill-loopen som håndterer input, oppdatering og tegning
    let mut state = MainState::new(ctx);
    event::run(ctx, event_loop, &mut state)
}
