# 0004. Recibir el contexto del estudiante en un token firmado por la institución

- **Estado:** Aceptado (implementación pendiente)
- **Fecha:** 2026-09-25

## Contexto

Además del catálogo, el motor necesita datos propios de cada estudiante: cursos
habilitados, grupos ya inscritos y tope de créditos. La institución ya los conoce al
cargar la vista del estudiante. Si esos datos viajaran sin protección desde el navegador,
el estudiante podría alterarlos (por ejemplo, agregarse un curso sin prerrequisitos).

## Decisión

Al cargar la vista, la institución emite un **token firmado** (JWT con EdDSA/Ed25519) con:

- `sub`: identificador del estudiante.
- Cursos habilitados, grupos inscritos y tope de créditos.
- `exp`: vencimiento corto (la duración de la sesión de matrícula).

El navegador envía el token en cada solicitud al gateway y al abrir el WebSocket. El
adaptador de entrada verifica la firma con la **llave pública** de la institución y
entrega a los casos de uso un `StudentContext` ya verificado.

## Consecuencias

- El motor sigue sin llamadas de salida y sin estado por estudiante.
- El mismo token autentica al estudiante en el WebSocket.
- La institución solo necesita firmar un JWT, algo estándar en cualquier stack; el SDK
  incluirá un ejemplo.
- Si la situación del estudiante cambia durante la sesión, el cambio se refleja cuando la
  institución emita un token nuevo.
- La verificación vive en los adaptadores de entrada; los casos de uso reciben el
  contexto y no saben de tokens.

## Alternativas consideradas

- **Contexto sin firmar desde el navegador:** el estudiante puede manipularlo.
- **El backend de la institución como intermediario:** cada solicitud pasa por sus
  servidores y el WebSocket queda partido entre dos sistemas.
- **Consulta del motor a la institución por estudiante:** contradice el ADR 0002.
