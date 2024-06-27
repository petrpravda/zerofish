from io import StringIO

import torch
import numpy as np
import chess
import chess.pgn

# Define constants
PIECE_TYPES = ['p', 'n', 'b', 'r', 'q', 'k']
BOARD_SIZE = 8 * 8  # 64 squares
NUM_FEATURES = len(PIECE_TYPES) * BOARD_SIZE  # HalfKP representation

# Function to encode a single position in the HalfKP format
def encode_position(board):
    feature_vector = np.zeros(NUM_FEATURES, dtype=np.float32)

    # Encode piece positions
    for square in chess.SQUARES:
        piece = board.piece_at(square)
        if piece:
            piece_type = piece.symbol().lower()
            color_offset = 0 if piece.color == chess.WHITE else len(PIECE_TYPES)
            piece_index = PIECE_TYPES.index(piece_type.lower()) + color_offset
            feature_index = piece_index * BOARD_SIZE + square
            feature_vector[feature_index] = 1.0

    return feature_vector

# Example PGN data for illustration
pgn_data = """
[Event "F/S Return Match"]
[Site "Belgrade, Serbia JUG"]
[Date "1992.11.04"]
[Round "29"]
[White "Fischer, Robert J."]
[Black "Spassky, Boris V."]
[Result "1/2-1/2"]
 
1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3
O-O 9. h3 Nb8 10. d4 Nbd7 11. c4 c6 12. Nc3 Bb7 13. a3 Re8 14. Ba2 Bf8 15. Bg5
h6 16. Bh4 g6 17. Qc2 Bg7 18. Rad1 Qc7 19. b4 exd4 20. Nxd4 Ne5 21. cxb5 cxb5
22. Nf5 gxf5 23. exf5 Rac8 24. Re3 Nc4 25. Rg3 Nh5 26. f6 Nxg3 27. Bxg3 Bxf6
28. Qg6+ fxg6 29. Bxc4+ Qxc4 30. Nd5 Bxd5 31. Rd4 Qc1+ 32. Kh2 Qg5 33. h4 Qh5
34. Rd5 g5 35. Kg1 Re1+ 36. Kh2 Qg4 37. Rxg5+ hxg5 38. h5 Qxh5# 0-1
"""

# TextIO
text_io_pgn = StringIO(pgn_data)

# Parse the PGN data
pgn = chess.pgn.read_game(text_io_pgn)
board = pgn.board()

# Prepare data (features and labels)
positions = []
for move in pgn.mainline_moves():
    board.push(move)
    encoded_position = encode_position(board)
    positions.append(encoded_position)

# Convert to PyTorch tensor
positions_tensor = torch.tensor(positions)

print(positions_tensor.shape)  # For illustration, printing the shape of the tensor

