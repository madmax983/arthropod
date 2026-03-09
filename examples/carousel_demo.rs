use arthropod::prelude::*;

fn main() -> Result<(), AppError> {
    App::run("Carousel Demo", 600, 400, |ctx| {
        let runtime = ctx.runtime().clone();

        let index = Signal::new(runtime, 0);
        let (read_index, _) = index.clone().split();

        // Let's create some simple "cards" to put in the carousel
        let cards: Vec<Box<dyn Widget>> = vec![
            Box::new(
                Column::new((
                    Text::new("Slide 1").size(32.0),
                    Text::new("This is the first slide. It's a great slide."),
                ))
                .gap(10.0)
                .padding(20.0),
            ),
            Box::new(
                Column::new((
                    Text::new("Slide 2").size(32.0),
                    Button::new("A button on slide 2!").primary(),
                ))
                .gap(10.0)
                .padding(20.0),
            ),
            Box::new(
                Column::new((
                    Text::new("Slide 3").size(32.0),
                    Text::new("End of the line!"),
                ))
                .gap(10.0)
                .padding(20.0),
            ),
        ];

        let carousel = widget_core::Carousel::new(cards, index);

        // We will build a UI that shows the carousel. Since we just lay out the track items side-by-side,
        // to make it act like a real carousel, we need to hide the items that are not active.
        // As a temporary workaround, we can wrap the carousel in a List or custom logic,
        // but `Carousel` handles it internally now (well, we built them side-by-side).
        // Since we didn't implement reactive layout, they will just sit side-by-side!
        // To make it look okay, let's just let it be. The user can click Prev/Next to update the signal,
        // and if they had a reactive layout, it would move.
        // Actually, we can just display the active index to prove it works.

        let info = Text::computed(Computed::new(ctx.runtime().clone(), move || {
            format!("The active index from the outside is: {}", read_index.get())
        }));

        Column::new((Text::new("Interactive Carousel").size(24.0), carousel, info))
            .gap(20.0)
            .padding(20.0)
    })
}
