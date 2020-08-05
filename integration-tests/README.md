# Integration tests

This directory holds all the integration tests. Currently, they are not automatized — i.e. they’re
not directly used in the CI. They serve as application we can run to check everything is fine. They
serve a similar purpose as [luminance-examples], yet are more technical integration tests.

## How to add an integration test

Simply run `cargo new` with the name of the test you want to add in the right folder under
`integration-tests`. Then, please add to the CI a line that `cargo build` the integration test.

[luminance-examples]: ../luminance-examples
