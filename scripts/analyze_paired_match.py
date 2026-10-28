#!/usr/bin/env python3
"""Fixed-sample paired noninferiority analysis. No third-party dependencies.

The primary interval is a large-sample paired Student-t interval, not a finite-
sample exact guarantee for discrete scores. A bounded Hoeffding interval is also
reported. Never use repeated ordinary intervals as an early-stopping rule.
"""
import argparse
import collections
import csv
import json
import math
import pathlib
import statistics


def _beta_fraction(a, b, x):
    tiny, eps = 1e-300, 3e-14
    qab, qap, qam = a+b, a+1, a-1
    c = 1.0
    d = 1-qab*x/qap
    if abs(d) < tiny:
        d = tiny
    d = 1/d
    h = d
    for m in range(1, 501):
        m2 = 2*m
        aa = m*(b-m)*x/((qam+m2)*(a+m2))
        d = 1+aa*d
        if abs(d) < tiny:
            d = tiny
        c = 1+aa/c
        if abs(c) < tiny:
            c = tiny
        d = 1/d
        h *= d*c
        aa = -(a+m)*(qab+m)*x/((a+m2)*(qap+m2))
        d = 1+aa*d
        if abs(d) < tiny:
            d = tiny
        c = 1+aa/c
        if abs(c) < tiny:
            c = tiny
        d = 1/d
        change = d*c
        h *= change
        if abs(change-1) < eps:
            return h
    raise ArithmeticError("Incomplete-beta continued fraction failed to converge")


def _beta(x, a, b):
    if x <= 0:
        return 0.0
    if x >= 1:
        return 1.0
    bt = math.exp(math.lgamma(a+b)-math.lgamma(a)-math.lgamma(b)
                  + a*math.log(x)+b*math.log1p(-x))
    if x < (a+1)/(a+b+2):
        return bt*_beta_fraction(a, b, x)/a
    return 1-bt*_beta_fraction(b, a, 1-x)/b


def t_cdf(value, degrees):
    if degrees <= 0:
        raise ValueError("Positive degrees of freedom required")
    half_tail = .5*_beta(degrees/(degrees+value*value), degrees/2, .5)
    return 1-half_tail if value >= 0 else half_tail


def t_quantile(probability, degrees):
    if not .5 < probability < 1:
        raise ValueError("Upper-half probability required")
    lo, hi = 0.0, 1.0
    while t_cdf(hi, degrees) < probability:
        hi *= 2
    for _ in range(80):
        middle = (lo+hi)/2
        if t_cdf(middle, degrees) < probability:
            lo = middle
        else:
            hi = middle
    return (lo+hi)/2


def elo(score):
    if score <= 0:
        return "-infinity"
    if score >= 1:
        return "infinity"
    return 400*math.log10(score/(1-score))


def analyze(path, expected_games, margin=10, alpha=.05):
    if expected_games < 4 or expected_games % 2 or margin <= 0 or not 0 < alpha < .5:
        raise ValueError("Invalid fixed-sample analysis parameters")
    with pathlib.Path(path).open() as stream:
        rows = list(csv.DictReader(stream))
    if len(rows) != expected_games:
        raise ValueError(f"Need the complete fixed sample: {expected_games} games, got {len(rows)}")
    pairs = collections.defaultdict(list)
    wdl = [0, 0, 0]
    terminations = collections.Counter()
    for row in rows:
        if row['result'] not in ('1-0', '0-1', '1/2-1/2'):
            raise ValueError("Unfinished game")
        if row['candidate_color'] not in ('white', 'black'):
            raise ValueError("Invalid candidate color")
        score = .5 if row['result'] == '1/2-1/2' else float(
            (row['result'] == '1-0') == (row['candidate_color'] == 'white'))
        wdl[0 if score == 1 else 1 if score == .5 else 2] += 1
        terminations[row['termination']] += 1
        pairs[row['pair']].append((row, score))
    values = []
    for games in pairs.values():
        if len(games) != 2 or {g[0]['candidate_color'] for g in games} != {'white', 'black'}:
            raise ValueError("Each pair must contain both candidate colors")
        if games[0][0]['opening'] != games[1][0]['opening']:
            raise ValueError("Pair opening IDs differ")
        values.append(sum(g[1] for g in games)/2)
    if len(values) != expected_games//2:
        raise ValueError("Incorrect opening-pair count")
    if any('protocol' in name or 'cap' in name for name in terminations):
        raise ValueError("Invalid protocol or move-cap termination")
    n = len(values)
    mean = statistics.mean(values)
    variance = statistics.variance(values)
    standard_error = math.sqrt(variance/n)
    threshold = 1/(1+10**(margin/400))
    hoeffding_lower = mean-math.sqrt(math.log(1/alpha)/(2*n))
    critical = t_quantile(1-alpha, n-1)
    lower = mean-critical*standard_error if variance else None
    statistic = (mean-threshold)/standard_error if variance else None
    p_value = 1-t_cdf(statistic, n-1) if variance else None
    noninferior = lower > threshold if variance else hoeffding_lower > threshold
    return dict(method='Fixed-sample one-sided paired Student-t; large-sample approximation',
                games=expected_games, opening_pairs=n, wins=wdl[0], draws=wdl[1], losses=wdl[2],
                score=mean, relative_elo=elo(mean), elo_margin=margin, alpha=alpha,
                null_hypothesis=f'candidate relative Elo <= -{margin}', null_score_threshold=threshold,
                sample_pair_variance=variance, score_standard_error=standard_error,
                t_critical=critical, t_statistic=statistic, one_sided_p_value=p_value,
                primary_score_lower=lower, primary_elo_lower=elo(lower) if lower is not None else None,
                hoeffding_score_lower=hoeffding_lower, hoeffding_elo_lower=elo(hoeffding_lower),
                noninferiority_criterion_met=noninferior,
                degenerate_variance_fallback='Hoeffding only' if variance == 0 else None,
                pair_mean_histogram=dict(collections.Counter(str(v) for v in values)),
                terminations=dict(terminations))


def self_test():
    # Independent closed forms for Student-t with 1 and 2 degrees of freedom.
    for value in [-10, -2, -.5, 0, .5, 2, 10]:
        assert abs(t_cdf(value, 1)-(.5+math.atan(value)/math.pi)) < 1e-12
        assert abs(t_cdf(value, 2)-(.5+value/(2*math.sqrt(value*value+2)))) < 1e-12
    assert abs(t_quantile(.95, 1)-math.tan(.45*math.pi)) < 1e-10
    # Independent Simpson integration of the PDF at the confirmation df.
    degrees = 999
    critical = t_quantile(.95, degrees)
    coefficient = math.exp(math.lgamma((degrees+1)/2)-math.lgamma(degrees/2)) / math.sqrt(degrees*math.pi)
    def pdf(x):
        return coefficient*(1+x*x/degrees)**(-(degrees+1)/2)
    intervals = 20000
    h = critical/intervals
    integral = h/3*(pdf(0)+pdf(critical)+4*sum(pdf(i*h) for i in range(1, intervals, 2))
                    + 2*sum(pdf(i*h) for i in range(2, intervals, 2)))
    assert abs(.5+integral-.95) < 1e-9
    print(f"Closed-form and independent numerical checks passed; t(.95,999)={critical:.12f}")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('csv', nargs='?')
    parser.add_argument('--expected-games', type=int, default=2000)
    parser.add_argument('--elo-margin', type=float, default=10)
    parser.add_argument('--alpha', type=float, default=.05)
    parser.add_argument('--output')
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    if args.self_test:
        self_test()
    if args.csv:
        result = analyze(args.csv, args.expected_games, args.elo_margin, args.alpha)
        encoded = json.dumps(result, indent=2, allow_nan=False)+'\n'
        if args.output:
            output = pathlib.Path(args.output)
            with output.open('x') as stream:
                stream.write(encoded)
        print(encoded)
    elif not args.self_test:
        parser.error('Provide a CSV or --self-test')
