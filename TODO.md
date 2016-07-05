# Todo

- README and example docs
- Review all docs and put in more examples.

## Form Bodies

form! {
    "key" => "value",
    "uploaded_file" => HttpFormFile::new() ?
}

Use `Multipart` crate for this.

## Mocks

Create a example with a mocked chrono instance.

Provide a macro for easy creation of mocks with global status flag etc. and 
macro for actual switching logic?

noir_mock! 

mock_chrono!

