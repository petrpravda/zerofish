package org.javafish.board;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class BoardPositionTest {

    @BeforeEach
    void setUp() {
    }

    @Test
    public void repetitionNotAllowed() {
//        BoardPosition position = Fen.fromFenFree("""
//                 +---+---+---+---+---+---+---+---+
//                 |   |   |   |   |   | r | k |   |
//                 +---+---+---+---+---+---+---+---+
//                 | p |   |   |   | b | p | p | p |
//                 +---+---+---+---+---+---+---+---+
//                 |   |   |   |   |   |   |   |   |
//                 +---+---+---+---+---+---+---+---+
//                 |   |   |   |   | P |   | P |   |
//                 +---+---+---+---+---+---+---+---+
//                 |   |   | B |   |   | N |   | K |
//                 +---+---+---+---+---+---+---+---+
//                 |   |   |   |   |   | q |   |   |
//                 +---+---+---+---+---+---+---+---+
//                 |   | P |   |   |   |   |   | P |
//                 +---+---+---+---+---+---+---+---+
//                 | R |   |   |   |   |   |   | R |
//                 +---+---+---+---+---+---+---+---+
//
//                Fen: 5rk1/p3bppp/8/4P1P1/2B2N1K/5q2/1P5P/R6R b - - 0 24
//                Key: 36A81FCB2C515B28
//                Checkers:""");
//
//        position.doMove("f3f4");
//        position.doMove("h4h3");
//        position.doMove("f4f3");
//        position.doMove("h3h4");
//        assertFalse(position.getState().isRepetitionOrFifty(position));
//        position.doMove("f3f4");
//        position.doMove("h4h3");
//        position.doMove("f4f3");
//        position.doMove("h3h4");
//        assertFalse(position.getState().isRepetitionOrFifty(position));
//        position.doMove("f3f4");
//        assertFalse(position.getState().isRepetitionOrFifty(position));
//        position.doMove("h4h3");
//        assertTrue(position.getState().isRepetitionOrFifty(position));
    }

    // 1. e4 c6 2. d4 d5 3. exd5 cxd5 4. Nf3 Nf6 5. Nc3 e6 6. Bf4 Bb4 7. a3 Bxc3+ 8.
    //bxc3 Ne4 9. Bxb8 Rxb8 10. Qd3 Qc7 11. Rb1 Nxc3 12. Rb3 Ne4 13. Nd2 Nxd2 14. Qxd2
    //O-O 15. Be2 Bd7 16. O-O Rbc8 17. Rfb1 b6 18. c3 e5 19. Ba6 Rce8 20. f3 Ba4 21.
    //Rb4 Bc6 22. Re1 exd4 23. Rxd4 b5 24. Re3 Qa5 25. Rxe8 Rxe8 26. a4 Qxa6 27. axb5
    //Qxb5 28. Kf2 Qc5 29. f4 Bb5 30. g3 Re2+ 31. Qxe2 Bxe2 32. Kxe2 a5 33. Kd2 Qa3
    //34. Ke2 Qa2+ 35. Kf3 f5 36. g4 Qc2 37. Ke3 Qxc3+ 38. Rd3 Qe1+ 39. Kd4 Qb4+ 40.
    //Ke3 Qe4+ 41. Kd2 Qxf4+ 42. Kc3 Qb4+ 43. Kc2 Qxg4 44. Rd2 a4 45. Kc1 a3 46. Kb1
    //Qe4+ 47. Ka2 Qc4+ 48. Kb1 Qb3+ 49. Ka1 Qc3+ 50. Ka2 Qxd2+ 51. Kxa3 Qxh2 52. Kb4
    //Qc2 53. Ka5 Qb1 54. Ka4 Qb6 55. Ka3 Qb1 56. Ka4 Qb6 57. Ka3 Qb1 58. Ka4 1/2-1/2
}
