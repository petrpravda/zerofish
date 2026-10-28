"""Independent score/decision fixtures for the fixed paired analysis."""
import csv
import pathlib
import tempfile
import unittest
import analyze_paired_match as stats


class PairedAnalysisTest(unittest.TestCase):
    def write_match(self, directory, pairs):
        path = pathlib.Path(directory)/'synthetic.csv'
        with path.open('w') as out:
            writer = csv.DictWriter(out, fieldnames=['pair', 'opening', 'candidate_color', 'result', 'termination'])
            writer.writeheader()
            for pair, results in enumerate(pairs):
                for color, result in zip(['white', 'black'], results):
                    writer.writerow(dict(pair=pair, opening=f'synthetic-{pair}', candidate_color=color,
                                         result=result, termination='synthetic fixture'))
        return path

    def test_balanced_nonzero_variance_matches_known_score(self):
        # Pair scores .25/.5/.75/.5 repeated: exact mean .5, known variance.
        with tempfile.TemporaryDirectory() as directory:
            pairs = [('0-1', '1/2-1/2'), ('1/2-1/2', '1/2-1/2'),
                     ('1-0', '1/2-1/2'), ('1/2-1/2', '1/2-1/2')]*250
            result = stats.analyze(self.write_match(directory, pairs), 2000)
            self.assertEqual((result['wins'], result['draws'], result['losses']), (250, 1500, 250))
            self.assertEqual(result['score'], .5)
            self.assertAlmostEqual(result['sample_pair_variance'], 31.25/999)
            self.assertTrue(result['noninferiority_criterion_met'])
            self.assertGreater(result['primary_elo_lower'], -10)
            self.assertLess(result['primary_elo_lower'], 0)

    def test_complete_sample_required(self):
        with tempfile.TemporaryDirectory() as directory:
            path = self.write_match(directory, [('1/2-1/2', '1/2-1/2')])
            with self.assertRaises(ValueError):
                stats.analyze(path, 2000)

    def test_zero_variance_does_not_claim_exact_parity(self):
        with tempfile.TemporaryDirectory() as directory:
            path = self.write_match(directory, [('1/2-1/2', '1/2-1/2')]*1000)
            result = stats.analyze(path, 2000)
            self.assertEqual(result['score'], .5)
            self.assertIsNone(result['primary_elo_lower'])
            self.assertFalse(result['noninferiority_criterion_met'])
            self.assertEqual(result['degenerate_variance_fallback'], 'Hoeffding only')


if __name__ == '__main__':
    unittest.main()
