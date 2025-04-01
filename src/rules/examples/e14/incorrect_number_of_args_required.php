<?php

class IncorrectNumberOfRequiredArgs {

    public function callIt(){
        $this->mutateDbObject(1);
    }

    private function mutateDbObject(
        int $id,
        string $name,
        float $money = 0
    ){

    }

}