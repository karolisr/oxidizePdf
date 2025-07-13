## Software Development Guidelines

- Always act as an expert software architect in Rust
- Treat all warnings as errors
- Before pushing changes to origin, ensure all tests pass successfully
- Aim for 95% coverage of documentation, unit tests, and integration tests, with a minimum acceptable threshold of 80%

## Project Status - Session 13/07/2025

### Completed ✅
- **Release v0.1.2 exitoso**: Primera release oficial en GitHub con pipeline automatizado
- **Pipeline CI/CD completo**: Release, CI, y coverage funcionando perfectamente
- **Doctests corregidos**: 58 doctests pasando (referencias `oxidize_pdf_core` → `oxidize_pdf` corregidas)
- **Tests unitarios e integración**: 84 tests pasando correctamente (0 fallos)
- **Sistema dual de testing implementado**: 
  - CI/CD usa PDFs sintéticos para builds rápidos y consistentes
  - Desarrollo local puede usar 743 PDFs reales via fixtures/symbolic link
- **Eliminación de warnings**: Todos los warnings tratados como errores y corregidos
- **Fixture system**: Detección automática de fixtures, estadísticas y sampling
- **Property tests reparados**: UTF-8 handling, dimensiones floating point, operadores balanceados
- **Release automation**: Merge automático a main, publicación a crates.io, versionado independiente

### Estado Actual del Código
- **Test Coverage**: 43.42% (2443/5627 líneas) - Medido con tarpaulin
- **Tests**: 142 tests pasando (84 unit/integration + 58 doctests)
- **CI/CD**: Pipeline funcionando sin fallos ni timeouts
- **Warnings**: 0 warnings de compilación
- **Estructura**: Workspace multi-crate funcional y organizado
- **Release**: v0.1.2 publicada en GitHub y crates.io

### Objetivos de Coverage 🎯
- **Objetivo**: 95% coverage (80% mínimo aceptable)
- **Actual**: 43.42% - Necesita mejora significativa
- **Áreas de bajo coverage identificadas**:
  - CLI (0/137 líneas): Sin tests
  - API (0/48 líneas): Sin tests
  - Parser components: object_stream (0/46), page_tree (19/205)
  - Semantic modules: entity (0/6), marking (0/27)

### Arquitectura de Testing
1. **Sintéticos** (CI/CD): PDFs generados programáticamente, rápidos, consistentes
2. **Reales** (Local): 743 PDFs via symbolic link `tests/fixtures`, comprehensive testing
3. **Property-based**: Tests con proptest para edge cases y fuzzing
4. **Exclusiones**: Tests lentos marcados como `#[ignore]` para CI performance
5. **Coverage**: Integrado en CI con tarpaulin y reportes HTML

### Métricas de Calidad
- Tests unitarios: 84/84 ✅
- Tests de integración: incluidos en los 84 ✅ 
- Doctests: 58/58 ✅ (corregidos)
- Coverage: 43.42% ❌ (objetivo 95%)
- Pipeline: funcionando sin timeouts ✅
- Release: automatizado ✅