package sched.server;

import org.slf4j.LoggerFactory;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

/** Punto de entrada. Es el único lugar que conoce todas las implementaciones concretas. */
@SpringBootApplication
public class SchedServer {

    public static void main(String[] args) {
        // La JVM ajusta sus hilos (recolector, compilador, hilos virtuales) a las CPUs
        // detectadas; en un contenedor debe coincidir con su límite de CPU. La comparación
        // de rendimiento (bench/) verifica este valor.
        LoggerFactory.getLogger(SchedServer.class)
                .info("CPUs detectadas: cpus={}", Runtime.getRuntime().availableProcessors());
        SpringApplication.run(SchedServer.class, args);
    }
}
