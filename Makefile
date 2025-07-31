debug = true
test:
	./scripts/test.sh '$(option)' '$(debug)'

run:
	./scripts/run.sh '$(arg)'
	
clean:
	rm -f *.o *~ tmp*

.PHONY: test clean run
