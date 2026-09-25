# 0002. La institución empuja todos los datos; el motor nunca la consulta

- **Estado:** Aceptado
- **Fecha:** 2026-09-25
- **Reemplaza:** la versión inicial de este ADR (rol pasivo por consulta gRPC)

## Contexto

La primera versión planteaba dos roles: uno pasivo, en el que el motor consultaba a la
institución por cada estudiante, y uno activo, en el que la institución avisaba los
cambios de cupo. Consultar en cada solicitud acopla la latencia del motor a la de la
institución, y además la institución ya conoce todo lo necesario desde antes: el catálogo
del periodo y, al cargar la vista del estudiante, sus cursos habilitados.

## Decisión

La institución **empuja** toda la información y el motor no hace llamadas de salida hacia ella.

1. **Sincronización del catálogo (continua).** Mediante el SDK, la institución envía el
   catálogo completo (cursos, grupos, franjas y cupos) y después solo los cambios.
   Ver [ADR 0003](0003-replica-del-catalogo.md).
2. **Contexto del estudiante (por sesión).** Al cargar la vista del estudiante, la
   institución emite un token firmado con sus datos, que viaja en cada solicitud.
   Ver [ADR 0004](0004-contexto-del-estudiante-en-token-firmado.md).

Adaptadores de entrada involucrados:

| Adaptador | Servicio gRPC | Qué recibe |
|---|---|---|
| `adapters/grpc_catalog` | `CatalogService.LoadCatalog` | Carga completa del catálogo |
| `adapters/grpc_events` | `CatalogEventsService.SyncChanges` | Cambios incrementales, con acuse |

El contrato vive en `proto/institution/v1/` y es la base del SDK: la institución genera
el cliente en su propio lenguaje a partir de los `.proto`.

## Consecuencias

- El motor no tiene base de datos propia ni depende de la disponibilidad de la
  institución para responder: solo de que la réplica esté sincronizada.
- La institución debe construir el componente que detecta cambios de cupo y los envía.
  Para reducir ese esfuerzo se entregará un conector de referencia junto con el SDK.
- La confirmación final de matrícula sigue siendo responsabilidad de la institución.

## Alternativas consideradas

- **Consulta por estudiante (versión anterior):** acopla la latencia y la disponibilidad
  del motor a las de la institución.
- **WebSocket entre institución y motor:** habría que definir a mano el formato de los
  mensajes, el orden y los acuses. gRPC ya los da y genera el SDK en cualquier lenguaje.
  WebSocket se reserva para el tramo entre el motor y el navegador.
