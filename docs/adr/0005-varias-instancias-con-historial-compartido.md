# 0005. Compartir el catálogo entre instancias con un historial en NATS JetStream

- **Estado:** Aceptado
- **Fecha:** 2026-09-25

## Contexto

Cada instancia del motor mantiene una réplica del catálogo en memoria (ADR 0003). Al
desplegar varias instancias en contenedores, la institución envía los datos a una sola y
las demás quedan sin catálogo o desactualizadas.

## Decisión

Usar **NATS JetStream** (ya presente en el stack) como **historial compartido** del
catálogo, con **un solo escritor**.

- **Instancia de ingesta** (`SCHED_ROLE=ingest`, una sola): es la única a la que se conecta
  el SDK. Valida el orden de `seq` (`IngestState`) y escribe cada entrada en el stream
  `SCHED_CATALOG`: carga completa en bloques, cambios y avisos de pérdida de sincronía.
- **Todas las instancias** leen el stream en orden (`CatalogFollower`) y aplican las
  entradas a su réplica local (`ApplyLogEntry`). Mismas entradas, mismo orden, mismo
  catálogo. La de ingesta también arma así su réplica.
- **Compactación:** tras escribir una carga completa se purga lo anterior. El stream
  siempre empieza en la última carga completa, así que una instancia nueva solo lee esa
  carga y los cambios posteriores.
- **Disponibilidad:** `/ready` responde 200 solo cuando la réplica está al día. El
  balanceador y el orquestador no envían tráfico antes. La ingesta abre su gRPC solo
  cuando está lista, porque de su réplica sale el último `seq` aceptado.
- **Pérdida de sincronía:** si la ingesta detecta un hueco o se cierra la conexión con la
  institución, escribe `SyncLost` en el historial y todas las réplicas marcan sus
  respuestas como `stale`.
- **Instancias de consulta** (`SCHED_ROLE=query`): escalables detrás de un balanceador. No
  necesitan afinidad de sesión: el contexto del estudiante viaja en el token (ADR 0004).
- `SCHED_ROLE=all` combina ambos roles para desarrollo o despliegues pequeños.

En hexagonal: el puerto de salida `CatalogLog` tiene dos implementaciones, JetStream
(`adapters/nats_bus`) y en memoria (`adapters/in_memory`, para simulación y pruebas),
así que el flujo de ingesta es el mismo con una o con N instancias. Las entradas se
serializan con protobuf (`proto/engine/v1/catalog_log.proto`).

## Consecuencias

- Todas las instancias convergen al mismo catálogo; cada respuesta indica el `seq` usado.
- Entre instancias puede haber milisegundos de diferencia mientras se propaga un cambio.
- **Punto único de falla en la ingesta:** si se cae, las consultas siguen respondiendo con
  la última réplica (marcada `stale` si se perdió la sincronía) y el SDK reintenta. Un
  solo escritor evita coordinar el `seq` entre varios.
- En producción NATS debe correr en clúster de 3 nodos con almacenamiento persistente.
- En Docker Compose el balanceador es **Traefik**: descubre las réplicas por el socket de
  Docker y solo envía tráfico a los contenedores `healthy`, cuyo healthcheck consulta
  `/ready`. Así, escalar no requiere reconfigurar nada y una réplica nueva no recibe
  solicitudes hasta tener su réplica al día. Se descartó nginx porque su versión libre
  resuelve las IP solo al arrancar y no hace chequeos activos de salud. En Kubernetes,
  el Service y la sonda de readiness cumplen el mismo papel.

## Alternativas consideradas

- **Que el SDK envíe los datos a cada instancia:** la institución tendría que conocer la
  topología del despliegue.
- **Redis o Postgres compartidos:** más infraestructura, contradice el "sin base de datos
  propia" y cada consulta pasaría por la red en lugar de leer memoria.
- **Varios escritores:** exigiría consenso sobre el `seq`, con mucha más complejidad.
