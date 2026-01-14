use aether_core::Template;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;

fn benchmark_template_parse(c: &mut Criterion) {
    let content = r#"
        <!DOCTYPE html>
        <html>
        <head><title>{{AI:title}}</title></head>
        <body>
            <header>{{AI:header:html}}</header>
            <main>{{AI:content:html}}</main>
            <footer>{{AI:footer}}</footer>
            <script>{{AI:script:js}}</script>
        </body>
        </html>
    "#;

    c.bench_function("template_parse_medium", |b| {
        b.iter(|| Template::new(black_box(content)))
    });
}

fn benchmark_template_render(c: &mut Criterion) {
    let content = r#"
        <div>{{AI:slot1}}</div>
        <div>{{AI:slot2}}</div>
        <div>{{AI:slot3}}</div>
        <div>{{AI:slot4}}</div>
        <div>{{AI:slot5}}</div>
    "#;
    let template = Template::new(content);
    
    let mut injections = HashMap::new();
    injections.insert("slot1".to_string(), "Content 1".to_string());
    injections.insert("slot2".to_string(), "Content 2".to_string());
    injections.insert("slot3".to_string(), "Content 3".to_string());
    injections.insert("slot4".to_string(), "Content 4".to_string());
    injections.insert("slot5".to_string(), "Content 5".to_string());

    c.bench_function("template_render_5_slots", |b| {
        b.iter(|| template.render(black_box(&injections)))
    });
}

criterion_group!(benches, benchmark_template_parse, benchmark_template_render);
criterion_main!(benches);
