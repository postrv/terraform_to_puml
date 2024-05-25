# Terraform to PUML

This is a simple tool to convert Terraform files to PlantUML files.

It also supports chat-based edits via the OpenAI API.

In time, I hope to add visualisation for the PUML diagrams, 
and support for integrating whole terraform repositories, rather than just individual files.

### To use:

1. Clone the repo
2. Add a `.env` file and add your OpenAI API key as `OPENAI_API_KEY`, e.g., `OPENAI_API_KEY=sk-1234567890xxxxxxxxxxxxxxxxxxxx`
3. Run the Rust code with `cargo run`
4. Open Local Host 8080 in your browser http://localhost:8080/
5. Add your Terraform code to the left-hand side and click the button to convert it to PUML ![step1](images/step1.png)
6. The PUML code will appear on the right-hand side and the diagram will be rendered at the bottom of the page ![step2](images/step2.png)
7. You can then add the Terraform code to the editor pane and give instructions to the OpenAI API to make changes to the code ![step3](images/step3.png)
8. The changes to the PUML will be shown in the right-hand pane and the diagram will be rendered below

Thanks for reading!

