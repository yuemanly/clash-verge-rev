import { Box, SvgIcon, TextField, Theme, styled } from "@mui/material";
import Tooltip from "@mui/material/Tooltip";
import { ChangeEvent, useState } from "react";

import { useTranslation } from "react-i18next";
import matchCaseIcon from "@/assets/image/component/match_case.svg?react";
import matchWholeWordIcon from "@/assets/image/component/match_whole_word.svg?react";
import useRegularExpressionIcon from "@/assets/image/component/use_regular_expression.svg?react";

type SearchProps = {
  placeholder?: string;
  onSearch: (
    match: (content: string) => boolean,
    state: {
      text: string;
      matchCase: boolean;
      matchWholeWord: boolean;
      useRegularExpression: boolean;
    }
  ) => void;
};

export const BaseSearchBox = styled((props: SearchProps) => {
  const { t } = useTranslation();
  const [matchCase, setMatchCase] = useState(true);
  const [matchWholeWord, setMatchWholeWord] = useState(false);
  const [useRegularExpression, setUseRegularExpression] = useState(false);
  const [errorMessage, setErrorMessage] = useState("");

  const iconStyle = {
    style: {
      height: "24px",
      width: "24px",
      cursor: "pointer",
    } as React.CSSProperties,
    inheritViewBox: true,
  };

  const onChange = (e: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    props.onSearch(
      (content) => doSearch([content], e.target?.value ?? "").length > 0,
      {
        text: e.target?.value ?? "",
        matchCase,
        matchWholeWord,
        useRegularExpression,
      }
    );
  };

  const doSearch = (searchList: string[], searchItem: string) => {
    setErrorMessage("");
    return searchList.filter((item) => {
      try {
        let searchItemCopy = searchItem;
        if (!matchCase) {
          item = item.toLowerCase();
          searchItemCopy = searchItemCopy.toLowerCase();
        }
        if (matchWholeWord) {
          const regex = new RegExp(`\\b${searchItemCopy}\\b`);
          if (useRegularExpression) {
            const regexWithOptions = new RegExp(searchItemCopy);
            return regexWithOptions.test(item) && regex.test(item);
          } else {
            return regex.test(item);
          }
        } else if (useRegularExpression) {
          const regex = new RegExp(searchItemCopy);
          return regex.test(item);
        } else {
          return item.includes(searchItemCopy);
        }
      } catch (err) {
        setErrorMessage(`${err}`);
      }
    });
  };

  return (
    <Tooltip title={errorMessage} placement="bottom-start">
      <TextField
        hiddenLabel
        fullWidth
        size="small"
        autoComplete="off"
        variant="outlined"
        spellCheck="false"
        placeholder={props.placeholder ?? t("Filter conditions")}
        sx={{ input: { py: 0.65, px: 1.25 } }}
        onChange={onChange}
        InputProps={{
          sx: { pr: 1 },
          endAdornment: (
            <Box display="flex">
              <Tooltip title={t("Match Case")}>
                <div>
                  <SvgIcon
                    component={matchCaseIcon}
                    {...iconStyle}
                    aria-label={matchCase ? "active" : "inactive"}
                    onClick={() => {
                      setMatchCase(!matchCase);
                    }}
                  />
                </div>
              </Tooltip>
              <Tooltip title={t("Match Whole Word")}>
                <div>
                  <SvgIcon
                    component={matchWholeWordIcon}
                    {...iconStyle}
                    aria-label={matchWholeWord ? "active" : "inactive"}
                    onClick={() => {
                      setMatchWholeWord(!matchWholeWord);
                    }}
                  />
                </div>
              </Tooltip>
              <Tooltip title={t("Use Regular Expression")}>
                <div>
                  <SvgIcon
                    component={useRegularExpressionIcon}
                    aria-label={useRegularExpression ? "active" : "inactive"}
                    {...iconStyle}
                    onClick={() => {
                      setUseRegularExpression(!useRegularExpression);
                    }}
                  />{" "}
                </div>
              </Tooltip>
            </Box>
          ),
        }}
        {...props}
      />
    </Tooltip>
  );
})(({ theme }) => ({
  "& .MuiInputBase-root": {
    background: theme.palette.mode === "light" ? "#fff" : undefined,
    "padding-right": "4px",
  },
  "& .MuiInputBase-root svg[aria-label='active'] path": {
    fill: theme.palette.primary.light,
  },
  "& .MuiInputBase-root svg[aria-label='inactive'] path": {
    fill: "#A7A7A7",
  },
}));
