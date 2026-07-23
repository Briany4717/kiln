# ⟡ Amyst

> Un lenguaje de programación de propósito general, forjado en Rust.

Amyst es un lenguaje de programación construido desde cero siguiendo el libro
[*Crafting Interpreters*](https://craftinginterpreters.com/) de Robert Nystrom, implementado en Rust.

Este proyecto es tanto un ejercicio de aprendizaje sobre diseño de lenguajes e implementación de intérpretes,
como la base de un lenguaje que busca ser rápido, tan simple como el usuario desee y agradable de usar.

## Estado del proyecto

**En desarrollo activo.** Amyst todavía está tomando forma.

*La sintaxis y las funcionalidades pueden cambiar sin previo aviso*.

- [x] Lexer / Scanner
- [x] Parser (AST)
- [x] Evaluador de expresiones
- [x] Statements y variables
- [x] Resolución de variables y scopes
- [ ] Clases y herencia
- [ ] Compilador a bytecode
- [ ] Máquina virtual (VM)
- [ ] Garbage collector
- [ ] Librería estándar básica

## ¿Por qué Amyst?

Mi abuela me regaló alguna vez un anillo de amatista y mientras buscaba un buen nombre para el proyecto lo recordé.

La amatista es un cuarzo violeta cuyo nombre viene del griego *amethystos*, que significa literalmente
**"no ebrio"**: los antiguos griegos creían que la piedra protegía contra la embriaguez y traía claridad mental.
Es una buena metáfora para un lenguaje que aspira a ser claro, directo y sin ruido innecesario, 
tanto en su sintaxis como en cómo se siente escribir en él.

## Instalación

```bash
git clone https://github.com/Briany4717/amyst.git
cd amyst
cargo build --release
```

## Uso

```bash
# Ejecutar un script .amy
./target/release/amyst run script.amy

# Abrir el REPL
./target/release/amyst
```

## Ejemplo

```amyst
fn fib(n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

print fib(10);
```

> Nota: la sintaxis final de Amyst aún está en definición y puede diferir de este ejemplo.

## Roadmap

Este proyecto sigue las dos partes del libro *Crafting Interpreters*:

1. **`jlox` → `amyst-tree`**: intérprete tree-walking (fase actual)
2. **`clox` → `amyst-vm`**: compilador a bytecode + máquina virtual, orientado a rendimiento

## Contribuir

Por ahora este es un proyecto personal de aprendizaje, pero issues, sugerencias y discusiones son bienvenidas.

## Licencia

[MIT](LICENSE)

---

*Construido siguiendo [Crafting Interpreters](https://craftinginterpreters.com/) de Robert Nystrom.*
