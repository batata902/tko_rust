use std::{io::Write, fs::File, path::{Path, PathBuf}};
use walkdir::WalkDir;

pub struct SandboxDrafts {
    sandbox_key_prefix: &'static str,
    draft_readme: &'static str,
    md_draft: &'static str // Não está sendo usado aqui neste arquivo
}

// Implementação dos métodos da struct SandboxDrafts
impl SandboxDrafts {
    // Construtor
    pub fn new() -> Self {
        Self {
            sandbox_key_prefix: "user",
            draft_readme: r#"Escreva aqui as informações que você quer salvar, esse é o seu rascunho.
O texto abaixo é informativo e você pode apagar depois de aprender como usar os rascunhos.

## Como usar os rascunhos

- A chave do seu rascunho é o nome da pasta.
- O título do seu rascunho é carregado a partir da primeira linha desse arquivo \#
- Tudo que você fizer nos rascunhos também será rastreado pelo tko.

## Como criar seus próprios testes

Um teste é composto de um `input` (o texto que será fornecido para o programa) e um `output` (o texto que o programa deve retornar para esse input) e pode ter opcionalmente um `label` para facilitar a identificação do teste.

Seus casos de teste personalizados podem ser escritos diretamente aqui na descrição do problema dentro de um fenced code block com a linguagem `toml` ou em um arquivo `tests.toml` na pasta do problema. O TKO irá carregar automaticamente os testes quando a tarefa for aberta ou executada novamente.

Exemplo de teste para ler dois números, um por linha, e imprimir a soma e a subtração deles.

Se quiser habilitar esses casos de teste e ver funcionando, insira algo no input e no output.

```toml
# Exemplo de entrada em uma linha
[[tests]]
input = ''
output = ''

# Exemplo de entrada em múltiplas linhas
# [[tests]]
# input = '''
# 1
# 2
# '''
# output = '''
# 3
# 4
# '''
```"#,
            md_draft: r#"
Se a tarefa exigir um relatório, escreva ele aqui. Você pode usar markdown, imagens e o que mais quiser para criar um relatório bem completo."#
        }
    }

    // Retorna o nome do direório que será criado formatado, nesse caso, por exemplo: user_001
    pub fn format_draft_key(&self, draft_id: u32) -> String {
        format!("{}_{:03}", self.sandbox_key_prefix, draft_id)
    }

    pub fn find_max_numbered_key(&self, task_keys_only: Vec<String>) -> u32 {
        let mut numbered_keys: Vec<u32> = Vec::new();
        for key in task_keys_only {
            if key.starts_with(&format!("{}_", self.sandbox_key_prefix)) {
                let number = &key[self.sandbox_key_prefix.len() + 1..];
                match number.parse::<u32>() {
                    Ok(n) => numbered_keys.push(n),
                    Err(_) => continue
                }
            }
        }
        numbered_keys.iter().max().copied().unwrap_or(0)
    }

    // Carrega drafts no folder com a linguagem específicada
    pub fn load_drafts_only(folder: &Path, lang: &str, extra: Option<Vec<String>>) -> Vec<PathBuf> {
        let extra = extra.unwrap_or_else(|| Vec::new());
        let mut draft_list: Vec<PathBuf> = Vec::new();
        let mut allowed = extra.clone(); 
        if lang != "" {
            allowed.push(lang.to_string());
        }
        if allowed.contains(&"c".to_string()) {
            allowed.push("h".to_string());
        }
        if allowed.contains(&"cpp".to_string()) {
            allowed.push("h".to_string());
            allowed.push("hpp".to_string());
        }
        if !folder.is_dir() {
            return Vec::new();
        }
        for entry in WalkDir::new(folder) {
            let entry = entry.unwrap();
            let path = entry.path();

            let cut_root = match path.parent().and_then(|p | p.strip_prefix(folder).ok()) {
                Some(p) => p,
                None => continue
            }; 

            if cut_root.components().any(|comp| {
                let name = comp.as_os_str().to_string_lossy();
                name.starts_with('.') || name.starts_with('_')
                }) {
                    continue;
                } 

                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                        
                        if allowed.iter().any(|ext| file_name.ends_with(ext)) {
                            draft_list.push(path.to_path_buf());
                        }
                    }
                }

        }
        draft_list
    }

    // Escreve o draft
    pub fn create_sandbox_draft(&self, dir: PathBuf, title: String) {
        let path = dir.join("README.md");

        match File::create(path) {
            Ok(mut f) => {
                let content = format!("# {}\n\n{}", title, self.draft_readme);
                f.write_all(content.as_bytes()).unwrap();
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}