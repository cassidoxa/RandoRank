# TODO: automate debug building + test
import randorank as rr

import pytest

def test_scoring_new_period():
    new_test_period_10 = rr.MultiPeriod()
    new_test_race_10 = {'first_place': 1400,
                        'second_place': 1430,
                        'third_place': 1460,
                        'fourth_place': 1480,
                        'fifth_place': 1510,
                        'sixth_place': 1540,
                        'seventh_place': 1570,
                        'eighth_place': 1600,
                        'ninth_place': 1630,
                        'tenth_place': 1660}
    new_test_period_10.add_race(new_test_race_10)
    test_rankings_10 = new_test_period_10.rank()

    assert test_rankings_10['first_place']['rating'] > test_rankings_10['second_place']['rating']
    assert test_rankings_10['second_place']['rating'] > test_rankings_10['third_place']['rating']
    assert test_rankings_10['third_place']['rating'] > test_rankings_10['fourth_place']['rating']
    assert test_rankings_10['fourth_place']['rating'] > test_rankings_10['fifth_place']['rating']
    assert test_rankings_10['fifth_place']['rating'] > test_rankings_10['sixth_place']['rating']
    assert test_rankings_10['sixth_place']['rating'] > test_rankings_10['seventh_place']['rating']
    assert test_rankings_10['seventh_place']['rating'] > test_rankings_10['eighth_place']['rating']
    assert test_rankings_10['eighth_place']['rating'] > test_rankings_10['ninth_place']['rating']
    assert test_rankings_10['ninth_place']['rating'] > test_rankings_10['tenth_place']['rating']

    new_test_period_3 = rr.MultiPeriod()
    new_test_race_3 = {'first_place': 1400,
                        'second_place': 1500,
                        'third_place': 1600}
    new_test_period_3.add_race(new_test_race_3)
    test_rankings_3 = new_test_period_3.rank()

    assert test_rankings_3['first_place']['rating'] > test_rankings_3['second_place']['rating']
    assert test_rankings_3['second_place']['rating'] > test_rankings_3['third_place']['rating']
