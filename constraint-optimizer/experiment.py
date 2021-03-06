import yaml
import pydantic
from typing import List
import functools
import numpy as np

from ortools.linear_solver import pywraplp
import numpy as np

class House(pydantic.BaseModel):
    name: str
    capacity: int 

class Ballot(pydantic.BaseModel):
    name: str
    ranking: List[dict]

serialize = lambda model, x: model(**x)

def find_ranking(ranking: List[dict], house):
    for each_ranking in ranking:
        if each_ranking.get("name") == house:
            return each_ranking["weight"]

def load_weights_and_ballots(houses: list, ballots: list, size: int):
    # solver = pywraplp.Solver.CreateSolver('housing', 'CBC')
    solver = pywraplp.Solver.CreateSolver('CBC')

    # 2d array (ballots x individual weight/size of ballot)
    print(len(ballots))
    weights = np.zeros((len(ballots), size), dtype=float).tolist()
    options = np.zeros((len(ballots), size), dtype=float).tolist()

    for x, each_ballot in enumerate(ballots):
        for y, each_house in enumerate(houses):
            # weight = each_ballot.ranking.get(each_house.name)
            weight = find_ranking(each_ballot.ranking, each_house.name)
            weights[x][y] = weight


    # overwriting and generating options variables
    for x, ballots in enumerate(options):
        for y, option in enumerate(ballots):
            options[x][y] = solver.BoolVar(f'p[{x}, {y}]')

        # adding constraint so that each person has only one dorm
        solver.Add(solver.Sum(options[x][y] for y in range(size)) == 1)
    # adding constraint so that it'll never be over capacity
    for house_enum in range(size):
        solver.Add(solver.Sum(options[x][house_enum] for x in range(len(options))) <= houses[house_enum].capacity)

    weights = np.asarray(weights, dtype=float)

    return solver, weights, options

def normalize(x):
    total = sum([choice.get("weight") for choice in x.ranking])

    for choice in x.ranking:
        choice["weight"] /= total

    # assert sum([choice.get("weight") for choice in x.ranking]) == 100

    return x

def scale(x):
    largest = max([choice.get("weight") for choice in x.ranking])

    for choice in x.ranking:
        choice["weight"] *= 1/largest

    # assert sum([choice.get("weight") for choice in x.ranking]) == 100

    return x

if __name__ == "__main__":
    with open("input.yaml") as f:
        data = yaml.load(f)

    houses = list(map(functools.partial(serialize, House), data.get("houses")))
    ballots = list(map(normalize, map(functools.partial(serialize, Ballot), data.get("ballots"))))
    print(len(ballots))
    # ballots = list(map(scale, map(functools.partial(serialize, Ballot), data.get("ballots"))))
    # size = data.get("size")
    size = len(houses)


    solver, weights, options = load_weights_and_ballots(houses, ballots, size)
    # print(weights, options)
    solver.Maximize(1 +
                    solver.Sum(weights[x][y] * options[x][y]
                    for x in range(len(ballots))
                    for y in range(size)))
    solver.Solve()

    # print(options)
    choices = [0]*len(houses)

    total_weight = 0
    for x, ballot in enumerate(options):
        for y, option in enumerate(ballot):
            if option.solution_value() == 1:
                print(ballots[x].name, "has house", houses[y].name, "with weight", weights[x][y])
                preferences = list(reversed(sorted(ballots[x].ranking, key=lambda x: x.get("weight"))))

                count = -1
                prev = 1
                # print(houses)
                for pref in preferences:
                    if pref.get("weight") < prev:
                        count += 1
                        prev = pref.get("weight")
                    if pref.get("name") == houses[y].name:
                        break
                choices[count] += 1
                # print(preferences, count)
                
                total_weight += weights[x][y]

    print("Weight", total_weight, "out of", sum([max(weight) for weight in weights]))

    print(choices)
