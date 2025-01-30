ORIGIN_INC   := inc
ORIGIN_SRC   := src
ORIGIN_OUT   := out
ORIGINS_SVG  := diagram
DIRS_DEPS    := image pdf csv

VIEWER := okular

DIRS_INC    := $(foreach dir, \
				 $(shell find $(ORIGIN_INC)/ -type d), \
				 $(dir:$(ORIGIN_INC)/%=%))
DIRS_SRC    := $(shell find $(ORIGIN_SRC)/ -type d)
DIRS_SVG    := $(foreach dir, $(ORIGINS_SVG), $(shell find $(dir)/ -type d))
DIRS_OUT    := $(DIRS_SRC)

TARGET_DIRS := $(ORIGIN_OUT) $(foreach dir, $(DIRS_OUT) $(DIRS_SVG), $(ORIGIN_OUT)/$(dir))

SVGS        := $(foreach dir, $(DIRS_SVG), $(wildcard $(dir)/*.svg))

DEPS := $(foreach dir, $(DIRS_SRC), $(wildcard $(dir)/*)) \
		$(foreach dir, $(DIRS_DEPS), $(shell find $(dir)/ -type f)) \
		$(foreach dir, $(ORIGIN_INC) $(DIRS_INC), $(wildcard $(dir)/*)) \
		$(SVGS:%.svg=$(ORIGIN_OUT)/%.pdf_tex)

TOTAL_INC := $(ORIGIN_INC) $(DIRS_INC) $(DIRS_SRC) $(foreach dir, $(DIRS_SVG), $(ORIGIN_OUT)/$(dir)) $(DIRS_DEPS)

BASE_CREATE_STRING = \makeatletter\def\input@path{$(foreach dir, $(TOTAL_INC), {$(dir)})}\makeatother\input{${@:.pdf=.tex}}
CREATE_STR         = "$(BASE_CREATE_STRING)"

targets := $(foreach file, $(wildcard *.tex), $(file:.tex=.pdf))

FAST=0

.PHONY: release debug test clean default view fast

default: release

fast: FAST=1
fast: release

release: $(ORIGIN_OUT)/.buildrelease $(targets) | $(TARGET_DIRS)

debug: CREATE_STR = "\newcommand*{\DEBUG}{}$(BASE_CREATE_STRING)"
debug: $(ORIGIN_OUT)/.builddebug $(targets) | $(TARGET_DIRS)

$(ORIGIN_OUT)/.build%: | $(TARGET_DIRS)
	find $(ORIGIN_OUT)/ -type f -exec rm {} \;
	rm -f *.pdf
	touch $@

view: release
	$(VIEWER) $(targets) &>/dev/null &

test:
	@echo SVGS        := $(SVGS)
	@echo ORIGIN_INC  := $(ORIGIN_INC)
	@echo ORIGIN_SRC  := $(ORIGIN_SRC)
	@echo ORIGIN_OUT  := $(ORIGIN_OUT)
	@echo DIRS_DEPS   := $(DIRS_DEPS)
	@echo DIRS_INC    := $(DIRS_INC)
	@echo DIRS_SRC    := $(DIRS_SRC)
	@echo DIRS_OUT    := $(DIRS_OUT)
	@echo TARGET_DIRS := $(TARGET_DIRS)
	@echo DEPS        := $(DEPS)
	@echo targets     := $(targets)

$(TARGET_DIRS):
	mkdir -p $@

%.pdf: %.tex $(DEPS) | $(TARGET_DIRS)
	pdflatex --output-dir=$(ORIGIN_OUT) $(CREATE_STR)
ifeq ($(FAST), 0)
	bibtex $(ORIGIN_OUT)/$(@:.pdf=.aux)
	pdflatex --output-dir=$(ORIGIN_OUT) $(CREATE_STR)
	pdflatex --output-dir=$(ORIGIN_OUT) $(CREATE_STR)
endif
	mv $(ORIGIN_OUT)/$@ .

$(ORIGIN_OUT)/%.pdf_tex: %.svg | $(TARGET_DIRS) # Can not be parallel :'(
	inkscape -D -z --file=$^ --export-pdf=$(@:_tex=) --export-latex

clean:
	rm -rf $(ORIGIN_OUT)/*
	rm -f $(ORIGIN_OUT)/.build*

