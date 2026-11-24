//! The answers of sc-app3's pump about the Dirt of an element of a plugin:
//! the outcome of each start, and the sounds that the pump drops.

#[cfg(test)]
mod tests {
    use rosc::OscType::String as Text;
    use uuid::Uuid;

    use crate::Reply;
    use crate::reply::custom::{Node, NodeFailure};
    use crate::testing::assert_wire;

    const PLUGIN: &str = "0b4e1a2c-9d7f-4e3a-8b6c-1f2e3d4c5b6a";

    const ELEMENT: &str = "5f0a3c9e21d47";

    fn uuid(text: &str) -> Uuid {
        Uuid::try_parse(text).expect("a uuid")
    }

    fn text(value: &str) -> rosc::OscType {
        Text(value.to_owned())
    }

    #[test]
    fn the_answers_of_a_start_and_of_a_drop_have_their_wire_form() {
        let started = Node {
            plugin: uuid(PLUGIN),
            id: ELEMENT.to_owned(),
        };
        assert_wire(
            &Reply::LoadDirtSuccess(started),
            "/loadDirt.success",
            vec![text(PLUGIN), text(ELEMENT)],
        );
        let refused = NodeFailure {
            plugin: uuid(PLUGIN),
            id: ELEMENT.to_owned(),
            reason: "No bus is left for the Dirt.".to_owned(),
        };
        assert_wire(
            &Reply::LoadDirtFailure(refused),
            "/loadDirt.failure",
            vec![
                text(PLUGIN),
                text(ELEMENT),
                text("No bus is left for the Dirt."),
            ],
        );
        let dropped = NodeFailure {
            plugin: uuid(PLUGIN),
            id: ELEMENT.to_owned(),
            reason: "The sound came too late.".to_owned(),
        };
        assert_wire(
            &Reply::DirtPlayFailure(dropped),
            "/dirt.play.failure",
            vec![
                text(PLUGIN),
                text(ELEMENT),
                text("The sound came too late."),
            ],
        );
    }
}
