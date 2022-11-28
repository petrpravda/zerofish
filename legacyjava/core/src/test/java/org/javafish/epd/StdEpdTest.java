package org.javafish.epd;

import org.junit.jupiter.api.Test;

import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.util.List;

class StdEpdTest {
    @Test
    public void parseEpds() {
        InputStream epdStream = StdEpd.class.getResourceAsStream("STS1-STS15_LAN_v3.epd");
        assert epdStream != null;
        List<String> epds = new BufferedReader(new InputStreamReader(epdStream)).lines().toList();
        System.out.println(epds.size());
    }
}
