# 0003. Mantener una réplica en memoria del catálogo, alimentada por la institución

- **Estado:** Aceptado
- **Fecha:** 2026-09-25
- **Reemplaza:** "Snapshot versionado de cupos" (versión inicial de este ADR)

## Contexto

Para validar y generar combinaciones en milisegundos, el motor necesita el catálogo y los
cupos a mano. El snapshot inicial solo guardaba cupos, no tenía carga inicial y no
detectaba eventos perdidos.

## Decisión

El motor mantiene una **réplica de solo lectura** del catálogo (`CatalogReplica`). La
institución es la fuente de verdad; el motor solo la replica en memoria.

1. **Carga completa.** Al conectarse, el SDK envía el catálogo entero con un número de
   secuencia `seq`, en bloques (`LoadCatalog`, streaming del cliente).
2. **Cambios incrementales.** Luego envía eventos (`SyncChanges`, streaming
   bidireccional), cada uno con su `seq`:
   - `SeatsChanged`: cupos disponibles de un grupo.
   - `SectionUpserted`: grupo nuevo o modificado.
   - `SectionRemoved`: grupo cerrado.
3. **Orden estricto.** Un evento se aplica solo si `seq == último + 1`.
   - `seq <= último`: duplicado; se ignora y se confirma (idempotente).
   - `seq > último + 1`: se perdió algo; la réplica queda **desactualizada** y el motor
     responde `ResyncRequired`. El SDK debe hacer una nueva carga completa.
4. **La versión la define la institución.** Cada respuesta de combinaciones incluye el
   `seq` del catálogo con que se calculó, un número con significado para ambos lados.
5. **Estado de sincronización.** Si el stream de cambios se cierra o hay un hueco, la
   réplica queda desactualizada. Vuelve a estar al día con una carga completa o, si el SDK
   se reconecta, cuando llega el `seq` siguiente sin huecos (nada se perdió). Mientras
   tanto el motor **sigue respondiendo** pero marca las respuestas con `stale = true`,
   para que el cliente lo advierta al estudiante. Antes de la primera carga, el motor
   responde que el catálogo no está disponible.
6. Con varias instancias, la réplica se alimenta de un historial compartido en NATS
   JetStream ([ADR 0005](0005-varias-instancias-con-historial-compartido.md)). Ese mismo
   historial es la fuente de los avisos de cupo que cada instancia empujará por WebSocket
   a sus estudiantes conectados.

## Consecuencias

- Las consultas son lecturas en memoria, sin I/O.
- El tamaño es manejable: miles de grupos ocupan unos pocos MB.
- La réplica puede ir milisegundos atrasada: dos estudiantes pueden ver el mismo último
  cupo. Es aceptable porque la matrícula la confirma la institución.
- Con varias instancias del motor, cada una mantiene su réplica a partir del historial
  compartido (ADR 0005).

## Alternativas consideradas

- **Consultar a la institución en cada solicitud:** latencia y disponibilidad acopladas.
- **Motor totalmente sin estado, con el catálogo dentro de cada solicitud:** mensajes
  enormes y ningún canal natural para avisar cambios de cupo.
