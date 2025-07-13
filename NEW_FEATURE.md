We are going to add a GUI for a very specific task.

You should use 'druid' as the ui framework.

The specification is as follows:

    There is a single screen in the UI.
    It is divided into two areas:
    - To the left there is a text display area which scrolls. The user cannot edit the text.
    - To the right there is a vertical column of six buttons. From top to bottom they are labeled:
      Top
      High
      Middle
      Low
      Bottom
      Calculate
    All the buttons should be gray except calculate which should be blue.

    The text are should start with the text "Current offset: 7". This is displaying a value of an internal counter called offset.
    Each time a button other than calculate is pressed, we do the following:
      - store (as a pair) the name of the button and the current offset
      - then, increment the current offset by 1
      - print the new current offset in the text area
    When calculate is pressed, we:
      - create a text file for input to reverse-rng using the /range feature. Each previous button press should create one line in the file, as follows:
         /range <button-name>, offset, 200, <button-low>, <button-high>
      - button-low and button-high are derived from the button-name as follows:
        - Bottom = 0, 4
        - Low = 3, 7
        - Mid = 6, 10
        - High = 9, 13
        - Top = 12, 16
      - once the file is created, we run reverse-rng using avx512 (see benchmark_avx_512.sh for an example) using this text file
      - The seed that is found should be displayed in the text area
      - then, we should run the same logic as routeFreshFile.sh except setting AthenaSeed=<the seed found in the previous step> and AthenaOffset=<current offset>
      - All the output from this run should appear in the text area
