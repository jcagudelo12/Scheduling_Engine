# 0006. Devolver las 5 mejores combinaciones con ramificación y poda y filtros del estudiante

- **Estado:** Aceptado (implementación pendiente)
- **Fecha:** 2026-09-29

## Contexto

El estudiante solo necesita ver unas pocas combinaciones: las **5 mejores**, no las primeras
que encuentre el solver ni decenas de ellas. Además quiere acotar la búsqueda con sus propias
condiciones (jornada, disponibilidad, créditos, sede, días libres) y decidir qué significa
"mejor" para él.

El informe técnico descartó las metaheurísticas porque no garantizan encontrar la solución
válida; la solución debe seguir siendo **exacta**.

## Decisión

### Filtros que envía el estudiante

| Filtro | Tipo | Regla |
|---|---|---|
| Materias que quiere ver | Obligatorio | **Todas** deben estar en cada combinación. Deben ser un subconjunto de sus cursos habilitados (token, ADR 0004) |
| Jornada **o** franjas horarias | Estricto, **uno a la vez** | Jornada: mañana, tarde o noche. Franjas: bloques de la semana en que puede asistir. Los grupos fuera de ellos no se consideran |
| Mínimo y máximo de créditos | Estricto | Valida la selección; el máximo no puede superar el tope institucional del token |
| Días libres | Estricto | Días sin ninguna clase |
| Máximo de horas por día | Estricto | Tope de horas de clase en un mismo día |
| Sede | Estricto | Solo grupos de las sedes elegidas |
| Prioridad por materia | Para alternativas | Orden en que se sacrifican materias si no hay solución con todas |

Los filtros estrictos se aplican **antes** de la búsqueda, descartando grupos, lo que reduce el
árbol de búsqueda.

### Qué es "mejor": criterios ordenados por el estudiante

El estudiante elige y **ordena por importancia** los criterios. Las combinaciones se comparan
en orden lexicográfico: gana la mejor en el primer criterio; si empatan, decide el segundo, y así.

- Menos huecos entre clases del mismo día.
- Menos días con clase.
- Más cupo disponible (menor riesgo de que un grupo se agote antes de matricular).
- Menos cambios de sede en un mismo día.

Si no elige, se usa un orden por defecto. "Más créditos" no aplica: con todas las materias
obligatorias, todas las combinaciones suman los mismos créditos.

### Algoritmo: ramificación y poda para los k mejores (k = 5)

1. **Orden de los cursos (MRV):** primero el curso con menos grupos disponibles.
2. **Orden de los grupos:** dentro de cada curso, primero los más prometedores según los
   criterios del estudiante, para encontrar pronto combinaciones buenas.
3. **Las k mejores encontradas** se mantienen en un *heap*.
4. **Poda por cota:** para cada rama se calcula el mejor puntaje que podría alcanzar en el caso
   más optimista; si no supera a la k-ésima mejor, la rama se descarta sin explorarla.
   Días usados, cupo y cambios de sede tienen cotas directas porque solo empeoran al agregar
   grupos; los huecos usan una cota relajada, porque una clase nueva puede llenar un hueco.

El resultado es **exacto**: son las k mejores combinaciones posibles, no una aproximación.

### Alternativas cuando no hay solución

Si no existe ninguna combinación con todas las materias (por choques, filtros o falta de
cupo), el motor busca alternativas **quitando primero las materias de menor prioridad**,
siempre respetando el mínimo de créditos. La respuesta indica qué materia se quitó y por qué.

## Consecuencias

- El catálogo necesita **créditos por curso** y **sede por grupo**: cambia el contrato
  `proto/institution/v1/` y el SDK de la institución.
- Los filtros viajan en el cuerpo de la solicitud; los límites institucionales (cursos
  habilitados, tope de créditos) en el token firmado. El motor valida los primeros contra los
  segundos.
- El costo por solicitud depende del orden de criterios que elija el estudiante; se medirá con
  `bench/` para cada orden por defecto.
- Se implementa en Rust y en Java para mantener la comparación.

## Alternativas consideradas

- **Enumerar todas, puntuar y ordenar:** exacta pero explora todo el árbol; se usará solo como
  referencia en pruebas para validar que la poda no pierde soluciones.
- **Beam search:** muy rápida, pero no garantiza las 5 mejores.
- **Pesos numéricos por criterio:** más flexible, pero difícil de entender en pantalla y de
  explicar; el orden por importancia cubre el caso de uso.
- **Elegir un subconjunto de materias según el rango de créditos:** descartado por decisión
  del producto; el estudiante elige exactamente qué materias quiere ver.
