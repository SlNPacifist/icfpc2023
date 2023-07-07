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

## Submissions

Все отправки здесь: https://www.icfpcontest.com/dashboard

```
submit.sh problem-id solution-data
```
или
```
submit.sh problem-id path-to-solution-data
```

Решение должно быть валидным json. bash не очень хорошо дружит с кавычками, поэтому если хотим отправить что-то сложное, лучше положить в файл.

```
submit.sh 1 17
submit.sh 2 abc # Не норм, abc - не валидный json
submit.sh 2 "abc" # Не норм, bash съедает кавычки
submit.sh 2 '"abc"'
submin.sh 2 \"abc\"
```

## Folder structure

`solutions/*` - решения. Любимые языки разные, решений может быть много.

## Sample problems

До старта соревнования на сайте висят три задачи без описания. Они принимают в качестве решения любой валидный json, дают 100 баллов. Скорборд забит фиксированными данными, не обновляется.