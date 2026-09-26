// Prueba de carga con k6: estudiantes pidiendo combinaciones a una tasa fija.
//
// Los estudiantes salen de students.json (generado con semilla fija) y se recorren en el
// mismo orden en cada corrida, así los dos motores reciben exactamente las mismas solicitudes.
//
// Variables: TARGET (url del motor), RATE (solicitudes/s), DURATION, OUT (resumen JSON).
import http from 'k6/http';
import { check } from 'k6';
import exec from 'k6/execution';
import { SharedArray } from 'k6/data';

const TARGET = __ENV.TARGET || 'http://localhost:18080';
const RATE = parseInt(__ENV.RATE || '500');
const DURATION = __ENV.DURATION || '30s';

const students = new SharedArray('students', () =>
  JSON.parse(open('./students.json')).map((s) => JSON.stringify(s)),
);

export const options = {
  discardResponseBodies: true,
  scenarios: {
    students: {
      executor: 'constant-arrival-rate',
      rate: RATE,
      timeUnit: '1s',
      duration: DURATION,
      preAllocatedVUs: Math.max(50, RATE / 5),
      maxVUs: Math.max(200, RATE * 2),
    },
  },
  summaryTrendStats: ['avg', 'min', 'med', 'p(90)', 'p(95)', 'p(99)', 'max'],
};

export default function () {
  const body = students[exec.scenario.iterationInTest % students.length];
  const res = http.post(TARGET + '/combinations', body, {
    headers: { 'content-type': 'application/json' },
  });
  check(res, { 'status 200': (r) => r.status === 200 });
}

// Resumen en JSON para que bench/run.sh arme la tabla comparativa.
export function handleSummary(data) {
  return { [__ENV.OUT || 'summary.json']: JSON.stringify(data) };
}
