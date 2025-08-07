debug = true
test:
	./scripts/test_ir.sh

test_old:
	./scripts/test.sh '$(option)' '$(debug)'

run:
	./scripts/run.sh '$(arg)'
	
clean:
	rm -f *.o *~ tmp*

.PHONY: test clean run
