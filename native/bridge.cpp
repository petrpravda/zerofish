// SPDX-License-Identifier: GPL-3.0-or-later
// Zerofish NNUE-only bridge. Stockfish search and UCI loop are never invoked.
#include "stockfish/attacks.h"
#include "stockfish/position.h"
#include "stockfish/evaluate.h"
#include "stockfish/uci.h"
#include "stockfish/nnue/network.h"
#include "stockfish/nnue/nnue_accumulator.h"
#include <array>
#include <memory>
#include <mutex>
#include <stdexcept>
#include <limits>
using namespace Stockfish;
namespace Stockfish { Value scale_evaluation(Value, int, const Position&); }
namespace {
std::once_flag once;
std::unique_ptr<Eval::NNUE::Network> network;
std::unique_ptr<Eval::NNUE::AccumulatorCaches> emptyCaches;
thread_local std::string error;
constexpr int Failure = std::numeric_limits<int>::min();
void initialize() {
    std::call_once(once, [] {
        Attacks::init(); Position::init();
        network = std::make_unique<Eval::NNUE::Network>();
        Eval::NNUE::EvalFile file;
        network->load_internal(file);
        if (!file.current) throw std::runtime_error("Embedded Stockfish weights failed to load");
        emptyCaches = std::make_unique<Eval::NNUE::AccumulatorCaches>(*network);
    });
}
struct Context {
    Position position;
    std::array<StateInfo, 256> states{};
    std::array<Move, 256> moves{};
    std::unique_ptr<Eval::NNUE::AccumulatorStack> stack;
    std::unique_ptr<Eval::NNUE::AccumulatorCaches> cache;
    int base = 0, depth = 0;
    Context() : stack(std::make_unique<Eval::NNUE::AccumulatorStack>()),
        cache(std::make_unique<Eval::NNUE::AccumulatorCaches>(*network)) {}
    void rewind(int target) {
        if (target < base || target > depth) throw std::runtime_error("NNUE path mismatch");
        while (depth > target) {
            if (moves[depth] == Move::null()) position.undo_null_move();
            else { position.undo_move(moves[depth]); stack->pop(); }
            --depth;
        }
    }
};
}
#define API extern "C" __attribute__((visibility("default")))
API const char* jf_nnue_error() { return error.c_str(); }
API void* jf_nnue_create() {
    try { initialize(); return new Context(); }
    catch (const std::exception& e) { error=e.what(); return nullptr; }
}
API void jf_nnue_destroy(void* pointer) { delete static_cast<Context*>(pointer); }
API int jf_nnue_reset(void* pointer, const char* fen, int chess960, int base) {
    try {
        auto& c=*static_cast<Context*>(pointer);
        if (base<0 || base>=250) throw std::runtime_error("Invalid base ply");
        c.states[base]=StateInfo{};
        if (auto failure=c.position.set(fen, chess960!=0, &c.states[base])) throw *failure;
        c.base=c.depth=base; c.stack->reset(); return 0;
    } catch (const std::exception& e) { error=e.what(); return Failure; }
}
API int jf_nnue_push(void* pointer, int ply, int from, int to, int flag) {
    try {
        auto& c=*static_cast<Context*>(pointer); c.rewind(ply-1);
        if (ply>=250) throw std::runtime_error("NNUE stack overflow");
        Move m;
        if (flag==7) {
            m=Move::null(); c.position.do_null_move(c.states[ply]);
        } else {
            if (flag & 8) m=Move::make<PROMOTION>(Square(from),Square(to),PieceType(KNIGHT+(flag&3)));
            else if (flag==2 || flag==3) m=Move::make<CASTLING>(Square(from),Square(to));
            else if (flag==5) m=Move::make<EN_PASSANT>(Square(from),Square(to));
            else m=Move(Square(from),Square(to));
            auto& dirties=c.stack->push();
            c.position.do_move(m,c.states[ply],c.position.gives_check(m),dirties,nullptr,nullptr);
        }
        c.moves[ply]=m; c.depth=ply; return 0;
    } catch (const std::exception& e) { error=e.what(); return Failure; }
}
API int jf_nnue_raw(void* pointer, int ply) {
    try { auto& c=*static_cast<Context*>(pointer); c.rewind(ply);
        return network->evaluate(c.position,*c.stack,*c.cache);
    } catch (const std::exception& e) { error=e.what(); return Failure; }
}
API int jf_nnue_eval(void* pointer, int ply) {
    try {
        auto& c=*static_cast<Context*>(pointer); c.rewind(ply);
        Value raw=network->evaluate(c.position,*c.stack,*c.cache);
        // Cache only position-dependent scores in Zerofish's transposition table.
        int clock=c.position.state()->rule50;
        c.position.state()->rule50=0;
        Value scaled=scale_evaluation(raw,0,c.position);
        c.position.state()->rule50=clock;
        return UCIEngine::to_cp(scaled,c.position);
    } catch (const std::exception& e) { error=e.what(); return Failure; }
}
API int jf_nnue_full_internal(void* pointer, int ply) {
    try { auto& c=*static_cast<Context*>(pointer); c.rewind(ply);
        return scale_evaluation(network->evaluate(c.position,*c.stack,*c.cache),0,c.position);
    } catch (const std::exception& e) { error=e.what(); return Failure; }
}
// Canonical (serialized neuron order) accumulator export for differential tests.
// This diagnostic API is never called by the Rust engine's inference backend.
API int jf_nnue_accumulator(void* pointer, int ply, int perspective, i16* output) {
    try {
        if (perspective < 0 || perspective > 1) throw std::runtime_error("Invalid perspective");
        auto& c = *static_cast<Context*>(pointer); c.rewind(ply);
        network->evaluate(c.position, *c.stack, *c.cache);
        const auto& values = c.stack->latest().accumulation[perspective];
#if defined(USE_AVX512)
        constexpr int order[8] = {0,2,4,6,1,3,5,7};
#elif defined(USE_AVX2)
        constexpr int order[8] = {0,2,1,3,4,6,5,7};
#else
        constexpr int order[8] = {0,1,2,3,4,5,6,7};
#endif
        for (int base = 0; base < 1024; base += 64)
            for (int block = 0; block < 8; ++block)
                for (int lane = 0; lane < 8; ++lane)
                    output[base + order[block] * 8 + lane] = values[base + block * 8 + lane];
        return 0;
    } catch (const std::exception& e) { error=e.what(); return Failure; }
}

// Full-refresh timing oracle: discard only the two selected cached entries,
// retaining no piece/accumulator history for this position. Clearing all 256
// entries here would measure unrelated cache initialization instead of inference.
API int jf_nnue_reset_full(void* pointer, const char* fen, int chess960, int base) {
    int status = jf_nnue_reset(pointer, fen, chess960, base);
    if (status == Failure) return status;
    try {
        auto& c = *static_cast<Context*>(pointer);
        for (Color side : {WHITE, BLACK}) {
            bool queen = bool(c.position.pieces(~side, QUEEN));
            Square king = c.position.square<KING>(side);
            (*c.cache)[queen][king][side] = (*emptyCaches)[queen][king][side];
        }
        return 0;
    } catch (const std::exception& e) { error=e.what(); return Failure; }
}
