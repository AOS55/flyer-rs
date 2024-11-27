use crate::world::SimWorld;
use tiny_skia::Pixmap;

impl SimWorld {
    pub fn render(&mut self) -> Pixmap {
        match self.state.render_type.as_str() {
            "world" => self.render_world(),
            "aircraft" => self.render_aircraft(),
            "aircraft_fixed" => self.render_fixed_aircraft(),
            _ => {
                println!(
                    "{} not a recognized render type, using world render",
                    self.state.render_type
                );
                self.render_world()
            }
        }
    }

    fn render_world(&mut self) -> Pixmap {
        // Original world rendering code
        unimplemented!("World rendering to be implemented")
    }

    fn render_aircraft(&mut self) -> Pixmap {
        // Original aircraft rendering code
        unimplemented!("Aircraft rendering to be implemented")
    }

    fn render_fixed_aircraft(&mut self) -> Pixmap {
        // Original fixed aircraft rendering code
        unimplemented!("Fixed aircraft rendering to be implemented")
    }
}
