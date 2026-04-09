use std::fs;
use std::path::PathBuf;
 // supondo que teu código esteja em sandbox.rs
use down::sandbox_drafts::SandboxDrafts;

mod down;

fn main() {
    let base_dir = PathBuf::from("drafts");

    // garante que a pasta existe
    if !base_dir.exists() {
        fs::create_dir(&base_dir).expect("Erro ao criar pasta base");
    }

    // pega diretórios existentes
    let mut existing_dirs: Vec<String> = Vec::new();

    for entry in fs::read_dir(&base_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                existing_dirs.push(name.to_string());
            }
        }
    }
    let sandbox = SandboxDrafts::new();

    // encontra maior ID
    let max_id = sandbox.find_max_numbered_key(existing_dirs);

    // próximo ID
    let new_id = max_id + 1;
    let new_key = sandbox.format_draft_key(new_id);

    let new_draft_dir = base_dir.join(&new_key);

    // cria diretório
    fs::create_dir(&new_draft_dir).expect("Erro ao criar pasta do draft");

    println!("Criando rascunho: {}", new_key);

    // cria README.md
    sandbox.create_sandbox_draft(new_draft_dir.clone(), format!("Rascunho {}", new_id));

    println!("README criado com sucesso!");

    // 🔎 listar arquivos .py
    println!("\nArquivos .py encontrados:");

    let drafts_files = SandboxDrafts::load_drafts_only(
        &base_dir.clone(),
        "py",
        None
    );

    for file in drafts_files {
        println!("{}", file.display());
    }
}