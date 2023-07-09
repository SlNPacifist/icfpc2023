# ICFPC2023

Private repo for participating in icfpc 2023

## Links

https://www.icfpcontest.com/dashboard
https://github.com/SlNPacifist/icfpc2023

## Setup

Для отправки решений скриптом:
* скопируй токен из tg-канала в файл token
* установи [jq](https://jqlang.github.io/jq/)
  debian:
  ```
  sudo apt update
  sudo apt install jq
  ```

  macos:
  ```
  brew update
  brew install jq
  ```
Для оптимизации решений через ortools нужно его поставить. В контейнере `debian:bookworm-slim` можно сделать так

```shell
apt update
apt install -y python3 python3-pip python3.11-venv python3-numpy
python3 -m pip install --upgrade --user ortools

python3 -m venv ~/ortools/
~/ortools/bin/pip install --upgrade ortools
~/ortools/bin/python ./code/ortools/solve_lin_ass.py
```

## Submissions

Все отправки здесь: https://www.icfpcontest.com/dashboard

```
submit.sh problem-id path-to-solution-data
submit.sh 7 ./solutions/problem-7.json
```

## Folder structure

`code/*` - код решений. Любимые языки разные, решений может быть много.
`data/*` - исходные данные
`solutions/*` - оптимальные ответы
`solutions-xxx/*` - субоптимальные ответы
