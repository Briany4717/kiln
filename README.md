# ◎ Kiln

> Un lenguaje de programación de propósito general, forjado en Rust.

Kiln es un lenguaje de programación construido desde cero siguiendo el libro
[*Crafting Interpreters*](https://craftinginterpreters.com/) de Robert Nystrom, implementado en Rust.

Este proyecto es tanto un ejercicio de aprendizaje sobre diseño de lenguajes e implementación de intérpretes,
como la base de un lenguaje que busca ser rápido, simple y agradable de usar.

## Estado del proyecto

**En desarrollo activo.** Kiln todavía está en una etapa temprana de desarrollo y diseño. La sintaxis y las funcionalidades pueden cambiar sin previo aviso.

- [x] Lexer / Scanner
- [x] Parser (AST)
- [ ] Intérprete tree-walking
- [ ] Resolución de variables y scopes
- [ ] Clases y herencia
- [ ] Compilador a bytecode
- [ ] Máquina virtual (VM)
- [ ] Garbage collector
- [ ] Librería estándar básica

## ¿Por qué Kiln?

Un horno de alfarero (*kiln*) toma algo maleable y, con calor y tiempo, lo convierte en algo sólido y duradero.
Ese es el espíritu del proyecto: partir de un diseño simple e ir cociendo el lenguaje hasta que tome forma final.

## Instalación

```bash
git clone https://github.com/Briany4717/kiln.git
cd kiln
cargo build --release
```

## Uso

```bash
# Ejecutar un script .kiln
./target/release/kiln script.kiln

# Abrir el REPL
./target/release/kiln
```

## Ejemplo

```kiln
fn fib(n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

print fib(10);
```

> Nota: la sintaxis final de Kiln aún está en definición y puede diferir de este ejemplo.


## Roadmap

Este proyecto sigue las dos partes del libro *Crafting Interpreters*:

1. **`jlox` → `kiln-tree`**: intérprete tree-walking (fase actual)
2. **`clox` → `kiln-vm`**: compilador a bytecode + máquina virtual, orientado a rendimiento

## Contribuir

Por ahora este es un proyecto personal de aprendizaje, pero issues, sugerencias y discusiones son bienvenidas.

## Licencia

[MIT](LICENSE)

---

*Construido siguiendo [Crafting Interpreters](https://craftinginterpreters.com/) de Robert Nystrom.*