
pub struct BeeperFactory {}

impl BeeperFactory {
    pub fn new() -> ::Result<BeeperFactory> {
        Ok(BeeperFactory {})
    }

    pub fn with_beeper<F>(&mut self, f: F) -> ::Result<()>
    where
        F: FnOnce(&mut Beeper) -> ::Result<()>,
    {
        let mut beeper = Beeper::new()?;
        f(&mut beeper)?;
        beeper.close()?;

        Ok(())
    }
}

pub struct Beeper<'a> {
    beeping: bool,
    _derp: ::std::marker::PhantomData<&'a ()>,
}

impl<'a> Beeper<'a> {
    fn new() -> ::Result<Beeper<'a>> {
        Ok(Beeper {
            beeping: false,
            _derp: ::std::marker::PhantomData,
        })
    }

    pub fn set_beeping(&mut self, beeping: bool) -> ::Result<()> {
        if self.beeping != beeping {
            self.beeping = beeping;
        }
        Ok(())
    }

    fn close(mut self) -> ::Result<()> {
        Ok(())
    }
}
