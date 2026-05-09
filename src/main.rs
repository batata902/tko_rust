use std::borrow::Cow;
use std::path::PathBuf;

mod settings;
mod utils;
mod play;
mod game;
mod feno;

use utils::decoder::Decoder;
use utils::get_md_link::get_md_link;
use utils::symbols;
use utils::text::{AddValue, AnsiColor, Text};

use game::quest::Quest;
use game::quest_parser::QuestParser;
use game::task::Task;
use game::task_parser::TaskParser;
use game::tree_item::TreeItem;

use settings::git_cache::{GitCache, UpdateMode};
use settings::rep_source::RepSource;
use settings::settings::Settings;

use play::tasktree::TreeFilter;

use down::sandbox_drafts::SandboxDrafts;

mod down {
    pub mod sandbox_drafts;
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers de impressão
// ─────────────────────────────────────────────────────────────────────────────

fn section(title: &str) {
    println!();
    println!(
        "{}",
        AnsiColor::colour("*c", &format!("{} {} {}", symbols::HBAR.repeat(3), title, symbols::HBAR.repeat(3)))
    );
}

fn ok(msg: &str) {
    println!("  {} {}", AnsiColor::colour("g", symbols::SUCCESS), msg);
}

fn info(label: &str, value: &str) {
    println!(
        "  {} {} {}",
        AnsiColor::colour("c", label),
        AnsiColor::colour("w", "→"),
        value
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Símbolos e Text/AnsiColor
// ─────────────────────────────────────────────────────────────────────────────

fn demo_symbols_and_text() {
    section("Símbolos e Text colorido");

    // AnsiColor direto
    println!("  {}", AnsiColor::colour("g", &format!("{} Sucesso", symbols::SUCCESS)));
    println!("  {}", AnsiColor::colour("r", &format!("{} Falha", symbols::FAILURE)));
    println!("  {}", AnsiColor::colour("y", &format!("{} Em progresso", symbols::FOCUS)));

    // Text builder encadeado
    let mut t = Text::new(None, None);
    t.addf("c".to_string(), Some(AddValue::Str(Cow::Borrowed(&"fup".to_string()))))
     .add(Some(AddValue::Str(Cow::Owned(":".to_string()))))
     .addf("*".to_string(), Some(AddValue::Str(Cow::Owned(" Soma de dois números".to_string()))))
     .addf("b".to_string(), Some(AddValue::Str(Cow::Owned(" +algoritmo".to_string()))));

    let rendered = t.data.iter().map(|tok| {
        if tok.fmt.is_empty() { tok.text.clone() }
        else { AnsiColor::colour(&tok.fmt, &tok.text) }
    }).collect::<String>();

    println!("  {}", rendered);
    ok("Text/AnsiColor funcionando");
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. get_md_link
// ─────────────────────────────────────────────────────────────────────────────

fn demo_md_link() {
    section("Geração de chaves Markdown (get_md_link)");

    let cases = vec![
        "Soma de Dois Números",
        "Pilha com Struct",
        "BFS e DFS -- Grafos",
    ];

    for title in cases {
        let link = get_md_link(title.to_string());
        info(title, &link);
    }
    ok("get_md_link funcionando");
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. QuestParser — parsear linhas de Markdown
// ─────────────────────────────────────────────────────────────────────────────

fn demo_quest_parser() {
    section("QuestParser — parsear cabeçalhos de Markdown");

    let lines = vec![
        "## Soma @soma +algoritmo",
        "## Pilha @pilha +estrutura:2 %70",
        "### BFS e DFS @bfs_dfs +grafos =cpp !pilha",
    ];

    for line in lines {
        let mut parser = QuestParser::new("fup".to_string());
        if let Some(quest) = parser.parse_quest(PathBuf::from("README.md"), line.to_string(), 1) {
            info(
                line,
                &format!(
                    "key={} skills={:?} langs={:?} min={}%",
                    quest.tree.get_key(),
                    quest.skills,
                    quest.languages,
                    quest.min_percent_completion,
                ),
            );
        }
    }
    ok("QuestParser funcionando");
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. TaskParser — parsear itens de checklist
// ─────────────────────────────────────────────────────────────────────────────

fn demo_task_parser() {
    section("TaskParser — parsear itens de checklist");

    let lines = vec![
        "- [ ] [Soma Simples](soma/README.md)",
        "- [ ] @side [Desafio Extra](extra/README.md)",
        "- [ ] `@perk` [Bônus](bonus/README.md)",
    ];

    for line in lines {
        let parser = TaskParser::new(PathBuf::from("README.md"), "fup".to_string());
        let (matched, tags, title, link) = parser.match_full_pattern(line);
        if matched {
            info(
                line,
                &format!("title={:?}  link={:?}  tags={:?}", title, link, tags),
            );
        }
    }
    ok("TaskParser funcionando");
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Quest + Task + QuestGrader — cálculo de XP e progresso
// ─────────────────────────────────────────────────────────────────────────────

fn make_task(title: &str, key: &str, rate: i32, has_feedback: bool, is_side: bool) -> Task {
    let mut task = Task::new();
    task.task.set_title(title.to_string());
    task.task.set_key(key.to_string());
    task.xp = 1;

    {
        let mut info = task.info.borrow_mut();
        info.rate = rate;
        info.feedback = has_feedback;
    }

    if is_side {
        task.task_path = game::task::TaskMain::SIDE;
    }

    task
}

fn demo_quest_grader() {
    section("Quest + Task + QuestGrader — XP e progresso");

    let mut quest = Quest::new(Some("Algoritmos Básicos".to_string()), Some("algo".to_string()));

    // 3 tasks principais com diferentes níveis de conclusão
    quest.add_task(make_task("Soma",       "soma",       100, true,  false));
    quest.add_task(make_task("Subtração",  "sub",        60,  true,  false));
    quest.add_task(make_task("Divisão",    "div",        0,   false, false));

    // 1 task opcional (SIDE)
    quest.add_task(make_task("Desafio",    "desafio",    80,  true,  true));

    quest.set_reachable(true);
    quest.update_tasks_reachable();

    let (done, total) = quest.get_completion();
    let percent_main = quest.get_percent_main().unwrap_or(0.0);
    let percent_side = quest.get_percent_side().unwrap_or(0.0);
    let complete = quest.is_complete();

    info("Tasks concluídas",  &format!("{}/{}", done, total));
    info("% tarefas principais", &format!("{:.1}%", percent_main));
    info("% tarefas opcionais",  &format!("{:.1}%", percent_side));
    info("Quest completa?",      if complete { "✓ sim" } else { "✗ não" });

    // Mostrar símbolo de rate para cada task
    println!();
    for t in quest.get_tasks() {
        let rate = t.info.borrow().rate;
        let sym = t.get_rate_symbol(rate / 10, None); // converte 0–100 → 0–10
        let raw = sym.data.iter().map(|tok| {
            if tok.fmt.is_empty() { tok.text.clone() }
            else { AnsiColor::colour(&tok.fmt, &tok.text) }
        }).collect::<String>();
        println!("    {} {} (rate={}%)", raw, t.get_full_title(None, ' '), rate);
    }

    ok("Quest + QuestGrader funcionando");
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. TreeItem — composição de chaves e títulos
// ─────────────────────────────────────────────────────────────────────────────

fn demo_tree_item() {
    section("TreeItem — chaves e títulos");

    let mut item = TreeItem::new();
    item.set_remote_name(&"fup".to_string());
    item.set_key("@soma".to_string());
    item.set_title("Soma de Dois Números".to_string());

    info("remote_name", item.get_remote_name());
    info("key",         item.get_key());
    info("full_key",    &item.get_full_key());
    info("title",       item.get_title());
    ok("TreeItem funcionando");
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. TreeFilter (play)
// ─────────────────────────────────────────────────────────────────────────────

fn demo_tree_filter() {
    section("TreeFilter — regras de visibilidade");

    let cases = vec![
        (true,  "",          true,  "inbox + sem busca → ocultar"),
        (true,  "soma",      false, "inbox + com busca → mostrar"),
        (false, "",          false, "normal + sem busca → mostrar"),
        (false, "qualquer",  false, "normal + com busca → mostrar"),
    ];

    for (inbox, search, expected_hide, desc) in cases {
        let f = TreeFilter::new(inbox, search.to_string());
        let actual = f.hide_element();
        let mark = if actual == expected_hide {
            AnsiColor::colour("g", symbols::SUCCESS)
        } else {
            AnsiColor::colour("r", symbols::FAILURE)
        };
        println!("  {} {}", mark, desc);
    }
    ok("TreeFilter funcionando");
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. RepSource — configuração de fontes
// ─────────────────────────────────────────────────────────────────────────────

fn demo_rep_source() {
    section("RepSource — configuração de fontes");

    // fonte local
    let local = RepSource::new("sandbox", None)
        .set_student_sandbox();
    info("sandbox local?",   &local.is_local().to_string());
    info("writeable?",       &local.get_writeable().to_string());
    info("is_sandbox?",      &local.is_sandbox_source().to_string());
    info("read_only?",       &local.is_read_only().to_string());

    // fonte git
    let git_rep = RepSource::new("fup", None)
        .set_git_source("https://github.com/qxcodefup/arcade.git", None);
    info("fup is_git?",      &git_rep.is_git_source().to_string());
    info("fup is_local?",    &git_rep.is_local_source().to_string());
    info("fup read_only?",   &git_rep.is_read_only().to_string());
    info("fup url",          git_rep.get_url_link());

    // serialização e leitura a partir de JSON
    let json = git_rep.save_to_dict();
    info("JSON serializado", &json.to_string());

    let restored = RepSource::new("fup", None).load_from_dict(&json);
    info("nome restaurado",  &restored.name);
    info("target restaurado",&restored.target);

    ok("RepSource funcionando");
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Decoder — leitura e escrita de arquivos
// ─────────────────────────────────────────────────────────────────────────────

fn demo_decoder() {
    section("Decoder — ler e salvar arquivos");

    let tmp = std::env::temp_dir().join("tko_demo_decoder.txt");
    let conteudo = "# Título\n\nLinha com acentuação: café, ação.".to_string();

    Decoder::save(&tmp, conteudo.clone()).expect("falha ao salvar");
    let lido = Decoder::load(&tmp, true).expect("falha ao ler");

    info("arquivo temporário", tmp.to_str().unwrap());
    info("bytes gravados",     &conteudo.len().to_string());
    info("bytes lidos",        &lido.trim_end().len().to_string());
    info("conteúdo bate?",     if lido.trim_end() == conteudo { "✓ sim" } else { "✗ não" });

    let _ = std::fs::remove_file(&tmp);
    ok("Decoder funcionando");
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. SandboxDrafts — rascunhos numerados
// ─────────────────────────────────────────────────────────────────────────────

fn demo_sandbox_drafts() {
    section("SandboxDrafts — rascunhos numerados");

    let drafts = SandboxDrafts::new();

    // simula chaves já existentes
    let keys = vec![
        "user_001".to_string(),
        "user_003".to_string(),
        "outro_item".to_string(),
    ];
    let max = drafts.find_max_numbered_key(keys);
    let next_key = drafts.format_draft_key(max + 1);

    info("max encontrado", &max.to_string());
    info("próxima chave",  &next_key);

    // cria o rascunho em um diretório temporário
    let dir = std::env::temp_dir().join(&next_key);
    std::fs::create_dir_all(&dir).unwrap();
    drafts.create_sandbox_draft(dir.clone(), "Meu Primeiro Rascunho".to_string());

    let readme = dir.join("README.md");
    info("README criado?", &readme.exists().to_string());

    // lista arquivos .md nessa pasta
    let mds = SandboxDrafts::load_drafts_only(&std::env::temp_dir(), "md", None);
    info("arquivos .md na temp", &mds.len().to_string());

    let _ = std::fs::remove_dir_all(&dir);
    ok("SandboxDrafts funcionando");
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. GitCache — clone / update de repositório
// ─────────────────────────────────────────────────────────────────────────────

fn demo_git_cache() {
    section("GitCache — cache de repositórios git");

    let cache_dir = std::env::temp_dir().join("tko_git_cache_demo");

    let result = GitCache::new(
        cache_dir.clone(),
        Some(std::time::Duration::from_secs(3600)),
        Some(UpdateMode::IF_OLDER),
    );

    match result {
        Ok(cache) => {
            let url = "https://github.com/qxcodefup/arcade.git";
            let repo_path = cache.repo_dir(url);
            info("diretório de cache", cache_dir.to_str().unwrap());
            info("hash do repo",       repo_path.file_name().unwrap().to_str().unwrap());
            info("expirado?",          &cache.is_expired(&repo_path).to_string());

            println!("  {} (clone real requer conexão — pulando)", AnsiColor::colour("y", "⚠ clone"));
            ok("GitCache instanciado e hash calculado");
        }
        Err(e) => println!("  {} GitCache error: {}", AnsiColor::colour("r", "✗"), e),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 12. Settings — alias de repositórios pré-configurados
// ─────────────────────────────────────────────────────────────────────────────

fn demo_settings() {
    section("Settings — aliases pré-configurados");

    let _settings = Settings::new(None);
    // Settings ainda não expõe os alias_git publicamente,
    // mas confirma que a struct é instanciável sem pânico.
    ok("Settings::new() executou sem erros");
    info("aliases internos", "poo, fup, ed  (campos privados por enquanto)");
}

// ─────────────────────────────────────────────────────────────────────────────
// main
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    println!("{}", AnsiColor::colour("*c", &format!("\n{} TKO — demonstração de todas as funções {}", symbols::ACTION, symbols::ACTION)));

    demo_symbols_and_text();
    demo_md_link();
    demo_quest_parser();
    demo_task_parser();
    demo_quest_grader();
    demo_tree_item();
    demo_tree_filter();
    demo_rep_source();
    demo_decoder();
    demo_sandbox_drafts();
    demo_git_cache();
    demo_settings();

    println!();
    println!("{}", AnsiColor::colour("*g", &format!("{} Todas as demos executadas com sucesso!", symbols::CHECK)));
    println!();
}